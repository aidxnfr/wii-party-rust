# AI-assisted Rust output backend for the original generated AX function.
# Compiler,not a verifier. Unknown syntax/type/callee raises;no placeholder instructions.
from pathlib import Path
import re,sys,shutil
from datetime import datetime
from prepare_ax import HERE,ROOT,prepare,c_parser,c_ast
TYPES={'bool':'bool','s8':'i8','u8':'u8','s16':'i16','u16':'u16','s32':'i32','u32':'u32','s64':'i64','u64':'u64','int':'i32','void':'()','UDSPInstruction':'u16'}
def dtype(n):
 if isinstance(n,c_ast.Typename):return dtype(n.type)
 if isinstance(n,c_ast.TypeDecl):return dtype(n.type)
 if isinstance(n,c_ast.IdentifierType):return TYPES[' '.join(n.names)]
 if isinstance(n,c_ast.PtrDecl):return '*'+dtype(n.type)
 raise ValueError(('type',type(n).__name__))
def prom(t):return 'i32' if t in ['bool','i8','u8','i16','u16'] else t
def common(a,b):
 a=prom(a);b=prom(b)
 if a==b:return a
 assert a in ['i32','u32','i64','u64'] and b in ['i32','u32','i64','u64'],(a,b)
 if a[1:]==b[1:]:return 'u'+a[1:]
 bigger=max([a,b],key=lambda t:int(t[1:]));smaller=b if bigger==a else a
 return bigger if bigger.startswith('u') or int(bigger[1:])>int(smaller[1:]) else 'u'+bigger[1:]
def cast(code,source,target):
 if source==target:return code
 if target=='bool':return '('+code+' != 0)'
 if source=='bool':return '('+code+' as '+target+')'
 return '('+code+' as '+target+')'
class Emit:
 def __init__(self,tree):
  self.fn=next(x for x in tree.ext if isinstance(x,c_ast.FuncDef));self.locals={};self.blocks={};self.labels={};self.serial=0
  self.resolve(self.fn.body,[{'s':'s','in':'in','cycles':'cycles','idle':'idle'}]);self.types={'cycles':'i32','idle':'*bool','s':'State','in':'Interpreter'}|self.locals
  source=(ROOT/'engine/include/wp/dsp_inline.h').read_text(encoding='utf-8')
  self.helpers={}
  for ret,name,args in re.findall(r'^WP_DSP_INLINE (\w+) (\w+)\(([^\n]+)\)',source,re.M):
   params=[];defaults=[]
   for arg in args.split(', '):
    decl,*default=arg.split(' = ');typ=decl.rsplit(' ',1)[0]
    params.append('State' if 'SDSP' in typ else TYPES[typ]);defaults.append(default[0] if default else None)
   self.helpers[name]=(TYPES[ret],params,defaults)
  self.context_helpers={'read_dmem','write_dmem','read_imem','read_reg','write_reg','extend_acc'}
  self.sdk={};self.constants={};header=(ROOT/'third_party/dolphin/Source/Core/Core/DSP/DSPCore.h').read_text(encoding='utf-8')
  clean=re.sub(r'//[^\n]*','',header)
  for typ,body in re.findall(r'enum\s*:\s*(u\d+)\s*\{(.*?)\}',clean,re.S):
   for name,value in re.findall(r'(\w+)\s*=\s*(0x[0-9a-fA-F]+|\d+)',body):self.constants['DSP_'+name]=(int(value,0),TYPES[typ])
  body=re.search(r'enum class StackRegister\s*\{(.*?)\}',clean,re.S).group(1);value=0
  for part in body.split(','):
   part=part.strip()
   if not part:continue
   name,*assigned=part.split('=')
   if assigned:value=int(assigned[0].strip(),0)
   self.constants['DSP_StackRegister_'+name.strip()]=(value,'i32');value+=1
 def resolve(self,n,scopes):
  if n is None:return
  if isinstance(n,c_ast.Compound):
   scopes=scopes+[{}]
   for child in n.block_items or []:self.resolve(child,scopes)
  elif isinstance(n,c_ast.Decl):
   old=n.name;name='v'+str(len(self.locals))+'_'+old;self.locals[name]=dtype(n.type);scopes[-1][old]=name;n.name=name
   self.resolve(n.init,scopes)
  elif isinstance(n,c_ast.ID):
   for scope in reversed(scopes):
    if n.name in scope:n.name=scope[n.name];break
  elif isinstance(n,c_ast.StructRef):self.resolve(n.name,scopes)
  elif isinstance(n,c_ast.FuncCall):
   for arg in n.args.exprs if n.args else []:self.resolve(arg,scopes)
  else:
   for _,child in n.children():self.resolve(child,scopes)
 def new(self,code):
  k=len(self.blocks);self.blocks[k]=code;return k
 def go(self,k):return 'block = '+str(k)+'; continue;'
 def jump(self,label):
  if label not in self.labels:self.labels[label]=self.new(None)
  return self.labels[label]
 def expr(self,n):
  if isinstance(n,c_ast.ID):
   name=n.name
   if name=='s':return 'ctx','State'
   if name=='in':return 'ctx.interpreter','Interpreter'
   if name in ['true','false']:return name,'bool'
   if name in self.constants:
    value,t=self.constants[name];return str(value)+t,t
   if name in self.locals:return name+'.assume_init_read()',self.locals[name]
   return name,self.types[name]
  if isinstance(n,c_ast.Constant):
   value=n.value;tail=re.search(r'[uUlL]+$',value);suffix=tail[0].lower() if tail else '';raw=value[:-len(suffix)] if suffix else value
   num=int(raw,16 if raw.lower().startswith('0x') else 8 if raw.startswith('0') and len(raw)>1 else 10)
   if 'll' in suffix:t='u64' if 'u' in suffix else 'i64'
   elif 'u' in suffix:t='u32' if num<=0xffffffff else 'u64'
   else:t='i32' if num<=0x7fffffff else ('u32' if raw.lower().startswith('0x') and num<=0xffffffff else 'i64')
   return str(num)+t,t
  if isinstance(n,c_ast.Cast):
   x,t=self.expr(n.expr);target=dtype(n.to_type);return cast(x,t,target),target
  if isinstance(n,c_ast.StructRef):
   a,t=self.expr(n.name);f=n.field.name
   if t=='State':
    mapping={'r':('(*ctx.regs)','Regs'),'pc':('(*ctx.pc)','u16'),'control_reg':('(*ctx.control)','u16'),'exceptions':('(*ctx.exceptions)','u8')}
    return mapping[f]
   if t=='Regs':
    return '('+a+').'+f,{'sr':'u16','cr':'u16','ar':'[u16]','ix':'[u16]','wr':'[u16]','st':'[u16]','ac':'[Accum]','ax':'[Aux]'}[f]
   if t in ['Accum','Aux']:return '('+a+').parts.'+f,('u32' if t=='Accum' and f=='h' else 'u16')
   raise ValueError(('field',t,f))
  if isinstance(n,c_ast.ArrayRef):
   a,t=self.expr(n.name);i,it=self.expr(n.subscript);assert t.startswith('[');return '('+a+')[('+i+') as usize]',t[1:-1]
  if isinstance(n,c_ast.UnaryOp):
   a,t=self.expr(n.expr)
   if n.op=='*':assert t.startswith('*');return '(*'+a+')',t[1:]
   if n.op=='!':return '(!'+cast(a,t,'bool')+')','bool'
   if n.op in ['-','+','~']:
    target=prom(t);a=cast(a,t,target)
    return ('('+a+').wrapping_neg()' if n.op=='-' else '!('+a+')' if n.op=='~' else a),target
   if n.op in ['p++','p--','++','--']:
    read,t=self.expr(n.expr);sign='wrapping_add' if '+' in n.op else 'wrapping_sub';v='('+read+').'+sign+'(1)';assign=self.assign(n.expr,v,t)
    return '{let old='+read+';'+assign+';'+('old' if n.op.startswith('p') else self.expr(n.expr)[0])+'}',t
   raise ValueError(('unary',n.op))
  if isinstance(n,c_ast.BinaryOp):
   a,at=self.expr(n.left);b,bt=self.expr(n.right);op=n.op
   if op in ['&&','||']:return '('+cast(a,at,'bool')+op+cast(b,bt,'bool')+')','bool'
   target=prom(at) if op in ['<<','>>'] else common(at,bt);a=cast(a,at,target);b=cast(b,bt,prom(bt) if op in ['<<','>>'] else target)
   if op in ['+','-','*']:return '('+a+').'+{'+':'wrapping_add','-':'wrapping_sub','*':'wrapping_mul'}[op]+'('+b+')',target
   if op in ['<<','>>']:return '('+a+').'+('wrapping_shl' if op=='<<' else 'wrapping_shr')+'(('+b+') as u32)',target
   return '('+a+op+b+')',('bool' if op in ['==','!=','<','>','<=','>='] else target)
  if isinstance(n,c_ast.TernaryOp):
   cond,ct=self.expr(n.cond);a,at=self.expr(n.iftrue);b,bt=self.expr(n.iffalse);target=common(at,bt)
   return '(if '+cast(cond,ct,'bool')+'{'+cast(a,at,target)+'}else{'+cast(b,bt,target)+'})',target
  if isinstance(n,c_ast.FuncCall):
   name=n.name.name;args=n.args.exprs if n.args else []
   if name=='__builtin_expect':return self.expr(args[0])
   if name=='__builtin_unreachable':return 'core::hint::unreachable_unchecked()','()'
   if name in self.helpers:
    ret,params,defaults=self.helpers[name];values=[]
    for i,param in enumerate(params):
     if i<len(args):a,t=self.expr(args[i])
     else:assert defaults[i] in ['false','true'];a,t=defaults[i],'bool'
     if param=='State':assert t=='State';values.append('ctx' if name in self.context_helpers else 'ctx.regs')
     else:values.append(cast(a,t,param))
    assert len(args)<=len(params);return 'ops::'+name+'('+','.join(values)+')',ret
   if name.startswith('sdk_'):
    meth=name[4:];ret,params=self.sdk_signature(meth);values=[]
    for i,arg in enumerate(args):
     a,t=self.expr(arg)
     if i==0:assert t in ['State','Interpreter'];values.append('ctx.state' if t=='State' else 'ctx.interpreter')
     else:values.append(cast(a,t,params[i-1]))
    assert len(args)==len(params)+1;self.sdk[meth]=(ret,params)
    return 'wphw_ax_sdk_'+meth+'('+','.join(values)+')',ret
   raise ValueError(('callee',name))
  raise ValueError(('expr',type(n).__name__,n.coord))
 def sdk_signature(self,name):
  if name in ['IsLoopEnd','IsIdleSkip']:header='DSPAnalyzer.h'
  elif name in ['StoreStack','PopStack','CheckExceptions']:header='DSPCore.h'
  else:header='Interpreter/DSPInterpreter.h'
  path=ROOT/'third_party/dolphin/Source/Core/Core/DSP'/header
  text=path.read_text(encoding='utf-8')
  if header=='DSPCore.h':text=text[text.index('struct SDSP'):text.index('class DSPCore',text.index('struct SDSP'))]
  matches=re.findall(r'\b(void|bool|u16)\s+'+name+r'\(([^)]*)\)',text);assert len(matches)==1,(name,matches)
  ret,params=matches[0];types=[]
  for p in params.split(','):
   p=p.strip()
   if p:typ=p.rsplit(' ',1)[0] if ' ' in p else p;types.append('i32' if typ=='StackRegister' else TYPES[typ])
  return TYPES[ret],types
 def assign(self,n,value,typ):
  if isinstance(n,c_ast.ID) and n.name in self.locals:return n.name+'.write('+cast(value,typ,self.locals[n.name])+')'
  left,lt=self.expr(n);return left+' = '+cast(value,typ,lt)
 def statement_expr(self,n):
  if isinstance(n,c_ast.Assignment):
   value,t=self.expr(n.rvalue)
   if n.op!='=':
    op=n.op[:-1];value,t=self.expr(c_ast.BinaryOp(op,n.lvalue,n.rvalue))
   return self.assign(n.lvalue,value,t)+';'
  return self.expr(n)[0]+';'
 def lower(self,n,next):
  if n is None:return next
  if isinstance(n,c_ast.Compound):
   for item in reversed(n.block_items or []):next=self.lower(item,next)
   return next
  if isinstance(n,c_ast.Decl):return self.new(self.assign(c_ast.ID(n.name),*self.expr(n.init))+';'+self.go(next)) if n.init else next
  if isinstance(n,c_ast.If):
   yes=self.lower(n.iftrue,next);no=self.lower(n.iffalse,next);v,t=self.expr(n.cond);return self.new('block=if '+cast(v,t,'bool')+'{'+str(yes)+'}else{'+str(no)+'};continue;')
  if isinstance(n,c_ast.Goto):return self.jump(n.name)
  if isinstance(n,c_ast.Label):
   dest=self.jump(n.name);target=self.lower(n.stmt,next);self.blocks[dest]=self.go(target);return dest
  if isinstance(n,c_ast.Return):v,t=self.expr(n.expr);return self.new('return '+cast(v,t,'i32')+';')
  if isinstance(n,c_ast.Switch):
   items=n.stmt.block_items;targets=[];default=next
   for item in reversed(items):
    assert isinstance(item,(c_ast.Case,c_ast.Default)),type(item).__name__
    for stmt in reversed(item.stmts or []):next=self.lower(stmt,next)
    if isinstance(item,c_ast.Default):default=next
    else:targets.append((self.expr(item.expr)[0],next))
   value,t=self.expr(n.cond);value=cast(value,t,prom(t));return self.new('block=match '+value+'{'+''.join(k+'=>'+str(v)+',' for k,v in reversed(targets))+'_=>'+str(default)+'};continue;')
  if isinstance(n,c_ast.EmptyStatement):return next
  return self.new(self.statement_expr(n)+self.go(next))
 def compact(self,entry):
  # Compiler-only branch-label forwarding and single-entry straight-line fusion.
  # Expressions and their source order are unchanged;never fuse through a join/condition.
  aliases={k:int(m[1])for k,v in self.blocks.items()if(m:=re.fullmatch(r'block = (\d+); continue;',v))}
  def target(k):
   path=[];seen=set()
   while k in aliases:
    assert k not in seen,('alias cycle',k)
    seen.add(k);path.append(k);k=aliases[k]
   for p in path:aliases[p]=k
   return k
  def rewrite(v):
   v=re.sub(r'block = (\d+); continue;',lambda m:'block = '+str(target(int(m[1])))+'; continue;',v)
   v=re.sub(r'\{(\d+)\}else\{(\d+)\};continue;$',lambda m:'{'+str(target(int(m[1])))+'}else{'+str(target(int(m[2])))+'};continue;',v)
   return re.sub(r'=>(\d+)(?=[,}])',lambda m:'=>'+str(target(int(m[1]))),v)
  entry=target(entry);self.blocks={k:rewrite(v)for k,v in self.blocks.items()if k not in aliases}
  def edges(v):
   out=[int(x)for x in re.findall(r'block = (\d+); continue;',v)]
   m=re.search(r'\{(\d+)\}else\{(\d+)\};continue;$',v)
   if m:out += [int(m[1]),int(m[2])]
   out += [int(x)for x in re.findall(r'=>(\d+)(?=[,}])',v)]
   return out
  incoming={k:0 for k in self.blocks}
  for v in self.blocks.values():
   for k in edges(v):incoming[k]+=1
  for k in list(self.blocks):
   if k not in self.blocks:continue
   while (m:=re.search(r'block = (\d+); continue;$',self.blocks[k])):
    to=int(m[1])
    if to==k or to==entry or incoming[to]!=1:break
    self.blocks[k]=self.blocks[k][:m.start()]+self.blocks.pop(to)
  assert all(k in self.blocks for v in self.blocks.values()for k in edges(v))
  return entry
 def output(self):
  terminal=self.new('core::hint::unreachable_unchecked();');entry=self.lower(self.fn.body,terminal);assert all(v is not None for v in self.blocks.values());entry=self.compact(entry)
  out=['// PRIVATE GENERATED AX SOURCE PORT;UNCOMPILED/UNINTEGRATED. Never commit or upload.','// Generated from original clean-gen/dsp/ax.cpp by emit_ax.py;source types/promotions preserved.','#[path="ops.rs"]mod ops;','use ops::Context;','extern "C"{']
  for name,(ret,params) in sorted(self.sdk.items()):out.append('fn wphw_ax_sdk_'+name+'(p:*mut core::ffi::c_void'+''.join(',a'+str(i)+':'+t for i,t in enumerate(params))+')->'+ret+';')
  out+=['}','#[no_mangle]pub unsafe extern "C" fn wprt_ax_run(ctx:*const Context,mut cycles:i32,idle:*mut bool)->i32{','let ctx=&*ctx;']
  for name,t in self.locals.items():out.append('let mut '+name+':core::mem::MaybeUninit<'+t+'>=core::mem::MaybeUninit::uninit();')
  out+=['let mut block='+str(entry)+'u32;','loop{match block{']
  for k,v in self.blocks.items():out.append(str(k)+'=>{'+v+'},')
  out+=['_=>core::hint::unreachable_unchecked(),','}}}'];return '\n'.join(out)+'\n'
