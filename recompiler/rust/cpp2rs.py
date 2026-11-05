# AI-assisted (Claude), PRIVATE tool: translate the recomp's generated C++ (a small, fixed dialect emitted by
# recompiler/ppc/emit.py) into equivalent Rust. Control flow (labels/goto/resume points) becomes a block-dispatch
# loop. Expressions keep the exact same operations; u32 arithmetic relies on wrapping (overflow-checks = false).
# usage: cpp2rs.py <module_name> <out.rs> <in1.cpp> [in2.cpp ...]
import re, sys

LABEL = re.compile(r'^([LR])_([0-9a-f]{8}):\s*$')
FUNC = re.compile(r'^void (f_\w+)\(wp::Cpu& c\) \{\s*$')

def strip_oracle(s):
    # Codex-era instrumentation in some generated files; the plain call is the original statement.
    s = re.sub(r'\{const uint32_t \w+=([^;]+);wp_stop_oracle::before\(c,\w+\);wp::call\(c,\w+\);wp_stop_oracle::after\(c,\w+\);\}',
               r'wp::call(c, \1);', s)
    s = re.sub(r'\{wp_stop_oracle::before\(c,[^;]+\);(.*?);wp_stop_oracle::after\(c,[^;]+\);\}', r'\1;', s)
    s = s.replace(' wp_stop_oracle::end(c,stop_trace_token);', '')
    s = s.replace('wp_lab_quantized::', 'wp::')
    return s

def expr(e, module):
    e = e.strip()
    e = re.sub(r'\bg_(\w+)_bases\[(\d+)\]', lambda m: f'wprt::modules::base(MODULE_ID, {m.group(2)})', e)
    e = re.sub(r'0x([0-9a-fA-F]+)ull \| (c\.\w+)', r'0x\1u64 | \2 as u64', e)
    e = re.sub(r'0x([0-9a-fA-F]+)u\b', r'0x\1u32', e)
    e = re.sub(r'(?<![\w.])(\d+)u\b', r'\1u32', e)
    e = re.sub(r'static_cast<float>\(', '(f32_of)(', e)
    e = re.sub(r'static_cast<uint32_t>\(', '(u32_of)(', e)
    e = re.sub(r'static_cast<int32_t>\(', '(i32_of)(', e)
    e = re.sub(r'std::fabs\(', 'wp::fabs(', e)
    e = re.sub(r'(wp::(?:load|store)_quantized\(c, [^)]*), q\)$', r'\1, &mut q)', e)
    e = e.replace('~', '!')
    # x86 keeps the FIRST operand's NaN; LLVM may swap commutative operands, GCC (the C++ recomp) keeps source order.
    e = re.sub(r'(c\.f\[\d+\]) \* (c\.f\[\d+\]|wp::force25\(c\.f\[\d+\]\))', r'wp_ordered_mul(\1, \2)', e)
    e = re.sub(r'(c\.f\[\d+\]) \+ (c\.f\[\d+\])', r'wp_ordered_add(\1, \2)', e)
    e = re.sub(r'\(f32_of\)\(', 'wp::to_f32(', e)
    e = re.sub(r'\(u32_of\)\(', 'wp::to_u32(', e)
    e = re.sub(r'\(i32_of\)\(', 'wp::to_i32(', e)
    e = re.sub(r'\bc\b(?=[,)])', 'c', e)
    e = re.sub(r'wp::(\w+)\(c([,)])', r'wp::\1(c\2', e)
    return e

class Fn:
    def __init__(self, name):
        self.name = name; self.lines = []

def parse(paths):
    fns = []; cur = None; depth = 0
    for p in paths:
        for raw in open(p, errors='replace'):
            line = raw.rstrip('\r\n')
            if not line.strip(): continue
            if cur is None:
                m = FUNC.match(line)
                if m: cur = Fn(m.group(1)); depth = 1
                continue
            if line == '}' and depth == 1:
                fns.append(cur); cur = None; continue
            cur.lines.append(line)
    return fns

def block_id(kind, addr):
    return addr  # L_ and R_ at one address are the same position

def translate_stmt(s, module, labels):
    """translate one simple statement (no braces). returns rust code."""
    s = strip_oracle(s.strip())
    if s in ('', ';') or s.startswith('WP_ENTER(') or s.startswith('[[maybe_unused]]'):
        return ''
    if s == 'return;': return 'return;'
    m = re.match(r'^goto ([LR])_([0-9a-f]{8});$', s)
    if m: return f'{{ pc = 0x{m.group(2)}; continue; }}'
    if s == 'WP_POLL(c);': return 'wp::poll(c);'
    m = re.match(r'^static const wp::HleFunction handler = wp::find_replacement\("(\w+)"\);$', s)
    if m: return f'wp::hle(c, "{m.group(1)}");'
    if s == 'handler(c);': return ''
    m = re.match(r'^for \(uint32_t k = (\d+); k < 32; k\+\+\) \{ c\.r\[k\] = wp::rd32\(ea\); ea \+= 4; \}$', s)
    if m: return f'for k in {m.group(1)}..32usize {{ c.r[k] = wp::rd32(ea); ea = ea.wrapping_add(4); }}'
    m = re.match(r'^for \(uint32_t k = (\d+); k < 32; k\+\+\) \{ wp::wr32\(ea, c\.r\[k\]\); ea \+= 4; \}$', s)
    if m: return f'for k in {m.group(1)}..32usize {{ wp::wr32(ea, c.r[k]); ea = ea.wrapping_add(4); }}'
    if s == 'c.ctr--;': return 'c.ctr = c.ctr.wrapping_sub(1);'
    m = re.match(r'^if \((.*)\) (goto [LR]_[0-9a-f]{8};|return;|wp::\w+\(c.*\);|f_\w+\(c\);)$', s)
    if m:
        return f'if {expr(m.group(1), module)} {{ {translate_stmt(m.group(2), module, labels)} }}'
    m = re.match(r'^(f_\w+)\(c\);$', s)
    if m: return f'{m.group(1)}(c);'
    m = re.match(r'^(.+?) = (.+);$', s)
    if m: return f'{expr(m.group(1), module)} = {expr(m.group(2), module)};'
    if s.endswith(';'): return expr(s[:-1], module) + ';'
    raise ValueError('unhandled statement: ' + s)

def translate_fn(fn, module):
    # split body into top-level items, handling brace blocks (if/else/switch) by depth tracking.
    out = []; resume = []; labels = set(); setjmp = []
    for l in fn.lines:
        m = LABEL.match(l.strip())
        if m: labels.add(int(m.group(2), 16))
    body = []
    i = 0; L = fn.lines
    # resume prologue
    while i < len(L):
        s = L[i].strip()
        if s.startswith('WP_ENTER(') or s.startswith('[[maybe_unused]]') or s.startswith('std::jmp_buf ') or not s:
            i += 1; continue
        if s.startswith('if (wp::g_resume_address != 0)'):
            i += 1
            while True:
                s = L[i].strip(); i += 1
                m = re.match(r'case (0x[0-9a-f]+)u: goto [LR]_([0-9a-f]{8});', s)
                if m: resume.append((m.group(1), m.group(2)))
                if s == '}' and L[i].strip() == '}': i += 1; break
            continue
        break
    entry_pc = 'ENTRY'
    blocks = [('ENTRY', [])]
    depth_stack = []
    def emit(code):
        blocks[-1][1].append(code)
    while i < len(L):
        raw = L[i]; s = strip_oracle(raw.strip()); i += 1
        if not s: continue
        m = LABEL.match(s)
        if m:
            a = int(m.group(2), 16)
            if depth_stack: raise ValueError(f'{fn.name}: label inside a block')
            if blocks[-1][0] != a:
                emit(f'{{ pc = 0x{a:08x}; continue; }}')
                blocks.append((a, []))
            continue
        m = re.match(r'^if \(setjmp\(wp_jump\[(\d+)\]\) == 0\) \{$', s)
        if m:
            site = m.group(1)
            save = L[i].strip(); i += 1   # wp::save_context(c, &wp_jump[k]);  or save_jump
            assert L[i].strip() == '} else {'; i += 1
            resume_call = L[i].strip(); i += 1
            assert L[i].strip() == '}'; i += 1
            kind = 'context' if 'save_context' in save else 'jump'
            # setjmp has no Rust equivalent. The save returns a token; a later load_context/long_jump on this
            # fiber unwinds (panic) back to this frame, which catches its own token and continues at the R_ label
            # that always follows the site, after resume_* (exactly the C++ else-branch).
            nxt = LABEL.match(strip_oracle(L[i].strip()))
            assert nxt, f'{fn.name}: setjmp site not followed by a label'
            if setjmp: raise ValueError(f'{fn.name}: two setjmp sites')
            setjmp.append((kind, int(nxt.group(2), 16)))
            emit(f'wp_token = wp::save_{kind}(c);')
            continue
        m = re.match(r'^if \((.*)\) \{$', s)
        if m:
            emit(f'if {expr(m.group(1), module)} {{'); depth_stack.append('if'); continue
        if s == '} else {':
            emit('} else {'); continue
        m = re.match(r'^switch \((.*)\) \{$', s)
        if m:
            emit(f'match {expr(m.group(1), module)} {{'); depth_stack.append('switch'); continue
        m = re.match(r'^case (0x[0-9a-f]+)u: (.*)$', s)
        if m:
            rest = m.group(2)
            parts = [p for p in re.split(r'(?<=;)\s+', rest) if p]
            emit(f'{m.group(1)}u32 => {{ ' + ' '.join(translate_stmt(p, module, labels) for p in parts) + ' }')
            continue
        m = re.match(r'^default: (.*)$', s)
        if m:
            emit(f'_ => {{ {translate_stmt(m.group(1), module, labels)} }}'); continue
        if s == '}':
            if not depth_stack: raise ValueError(f'{fn.name}: unbalanced')
            depth_stack.pop(); emit('}'); continue
        if s.endswith('{'):
            raise ValueError(f'{fn.name}: unhandled block: {s}')
        code = translate_stmt(s, module, labels)
        if code: emit(code)
    # emit function
    names = {b[0]: (f'0x{b[0]:08x}' if b[0] != 'ENTRY' else '0') for b in blocks}
    o = [f'#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, non_snake_case)]',
         f'pub fn {fn.name}(c: &mut Cpu) {{',
         '    let _enter = wp::enter();',
         '    let mut ea: u32 = 0; let mut q: u32 = 0; let mut pc: u32 = 0;']
    if resume:
        o.append('    if wp::resume_pending() {')
        o.append('        pc = match wp::take_resume_address() {')
        for a, lab in resume: o.append(f'            {a}u32 => 0x{lab},')
        o.append('            _ => { wp::bad_resume(c); return; }')
        o.append('        };')
        o.append('    }')
    body = ['    loop {', '        match pc {']
    for bid, code in blocks:
        key = '0' if bid == 'ENTRY' else f'0x{bid:08x}'
        body.append(f'            {key} => {{')
        for c in code: body.append('                ' + c)
        body.append('                return;')
        body.append('            }')
    body += ['            _ => unreachable!(),', '        }', '    }']
    if setjmp:
        kind, cont = setjmp[0]
        o.append('    let mut wp_token: u64 = 0;')
        o.append('    loop {')
        o.append('        let wp_run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {')
        o += ['        ' + b for b in body]
        o.append('        }));')
        o.append('        match wp_run {')
        o.append('            Ok(()) => return,')
        o.append(f'            Err(p) => {{ pc = wp::catch_resume(p, wp_token, 0x{cont:08x}); wp::resume_{kind}(c); }}')
        o.append('        }')
        o.append('    }')
    else:
        o += body
    o.append('}')
    return '\n'.join(o)

def descriptor(module_cpp, module):
    t=open(module_cpp,errors='replace').read()
    m=re.search(r'g_descriptor_%s = \{"%s", (\d+), (0x[0-9a-f]+)u, (\d+),'%(module,module),t)
    ident,sig,nsec=int(m.group(1)),m.group(2),int(m.group(3))
    def table(name):
        b=t.index('g_%s_%s[] = {'%(module,name)); e=t.index('};',b)
        return re.findall(r'\{(\d+), (0x[0-9a-f]+), (f_\w+)\}',t[b:e])
    fn=table('functions'); rs=table('resumes')
    o=[f'pub const MODULE_ID: u32 = {ident};', f'pub const SIGNATURE: u32 = {sig};', f'pub const SECTIONS: u32 = {nsec};',
       'pub static FUNCTIONS: &[(u32, u32, wprt::runtime::GuestFn)] = &[']+[f'    ({a}, {b}, {c}),' for a,b,c in fn]+['];',
       'pub static RESUMES: &[(u32, u32, wprt::runtime::GuestFn)] = &[']+[f'    ({a}, {b}, {c}),' for a,b,c in rs]+['];']
    return '\n'.join(o)

def main():
    args = sys.argv[1:]
    desc = None
    if args[0] == '--module-cpp': desc = args[1]; args = args[2:]
    module, out, *inputs = args
    fns = parse(inputs)
    res = [f'// GENERATED from the recomp C++ by cpp2rs.py (Claude). PRIVATE. module {module}',
           'use wprt::wp::{self, Cpu};', 'use crate::dol::*;', '#[allow(unused_imports)] use super::*;', '']
    # H1: emit the existing source-order arithmetic requirement locally, rather than calling absent runtime APIs.
    # The Windows x86-64 target has SSE2; the unchanged C++ function harness is the acceptance oracle.
    res.append('''// Port of generated C++ double multiply/add; constrain operand order (including NaN selection).
#[cfg(not(target_arch="x86_64"))] compile_error!("Ordered generated arithmetic requires the H1 x86_64 target");
#[cfg(target_arch="x86_64")] #[allow(dead_code)] #[inline(always)]
fn wp_ordered_mul(mut first:f64,second:f64)->f64 {
    unsafe { std::arch::asm!("mulsd {first}, {second}",first=inout(xmm_reg)first,second=in(xmm_reg)second,options(nomem,nostack,preserves_flags)); }
    first
}
#[cfg(target_arch="x86_64")] #[allow(dead_code)] #[inline(always)]
fn wp_ordered_add(mut first:f64,second:f64)->f64 {
    unsafe { std::arch::asm!("addsd {first}, {second}",first=inout(xmm_reg)first,second=in(xmm_reg)second,options(nomem,nostack,preserves_flags)); }
    first
}
''')
    ok = bad = 0
    for fn in fns:
        try:
            res.append(translate_fn(fn, module)); ok += 1
        except Exception as e:
            bad += 1; print('FAIL', fn.name, e, file=sys.stderr)
    if desc: res.append(descriptor(desc, module))
    open(out, 'w').write('\n'.join(res) + '\n')
    print(f'{ok} functions translated, {bad} failed', file=sys.stderr)

main()
