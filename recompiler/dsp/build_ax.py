#!/usr/bin/env python3
"""Turn the upstream-generated AX audio microcode (ax.cpp) into the Rust crate generated/dsp-ax.

usage: build_ax.py <upstream_dir> <ax.cpp> <out_crate_dir>
Same emitter as the accepted build (emit_ax.py), split into 256-block parts, plus the exception-check
fast path (identical result: SDSP::exceptions == 0 means CheckExceptions would return false).
"""
import os, re, shutil, sys
from pathlib import Path

up, ax_cpp, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
os.environ["WP_UPSTREAM_DIR"] = str(up); os.environ["WP_AX_CPP"] = str(ax_cpp)
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from prepare_ax import c_parser, prepare  # noqa: E402
from emit_ax import Emit  # noqa: E402

e = Emit(c_parser.CParser().parse(prepare())); text = e.output(); header = text[:text.index('#[no_mangle]')]
entry = int(re.search(r'let mut block=(\d+)u32;', text)[1]); size = 256
parts = {}
for k, code in e.blocks.items(): parts.setdefault(k // size, []).append((k, code))
lines = [header, 'enum Flow{Continue(u32),Return(i32)}', 'struct Frame{cycles:i32,']
for name, t in e.locals.items(): lines.append(name + ':core::mem::MaybeUninit<' + t + '>,')
lines += ['}']
for group, blocks in sorted(parts.items()):
    lines.append('unsafe fn part_' + str(group) + '(ctx:&Context,frame:*mut Frame,idle:*mut bool,mut block:u32)->Flow{loop{match block{')
    for k, code in blocks:
        code = re.sub(r'\bv\d+_[A-Za-z_]\w*\b', lambda m: '(*frame).' + m[0], code)
        code = re.sub(r'\bcycles\b', '(*frame).cycles', code)
        code = re.sub(r'\breturn ([^;]+);', lambda m: 'return Flow::Return(' + m[1] + ');', code)
        lines.append(str(k) + '=>{' + code + '},')
    lines += ['_=>return Flow::Continue(block),', '}}}']
lines += ['#[no_mangle]pub unsafe extern "C" fn wprt_ax_run(ctx:*const Context,cycles:i32,idle:*mut bool)->i32{', 'let ctx=&*ctx;let mut local=Frame{cycles,']
for name in e.locals: lines.append(name + ':core::mem::MaybeUninit::uninit(),')
lines += ['};let frame=&mut local as *mut Frame;let mut block=' + str(entry) + 'u32;', 'loop{let flow=match block/' + str(size) + '{']
for group in sorted(parts): lines.append(str(group) + '=>part_' + str(group) + '(ctx,frame,idle,block),')
lines += ['_=>core::hint::unreachable_unchecked(),', '};match flow{Flow::Continue(next)=>block=next,Flow::Return(cycles)=>return cycles}}}']
src = '\n'.join(lines) + '\n'
# Exception-check fast path (Dolphin's early-out): SDSP::exceptions sits at byte offset 92.
calls = src.count('wphw_ax_sdk_CheckExceptions(ctx.state)')
src = src.replace('wphw_ax_sdk_CheckExceptions(ctx.state)', 'ax_check_exceptions_fast(ctx.state)')
src += ('#[inline(always)] unsafe fn ax_check_exceptions_fast(p:*mut core::ffi::c_void)->bool'
        '{if *(p as *const u8).add(92)==0{false}else{wphw_ax_sdk_CheckExceptions(p)}}\n')
out.mkdir(parents=True, exist_ok=True)
(out / 'ax.chunked.rs').write_text(src, encoding='utf-8', newline='\n')
for f in ('ops.rs', 'register_constants.rs', 'status_constants.rs'):
    shutil.copyfile(HERE / 'runtime' / f, out / f)
(out / 'Cargo.toml').write_text('''[package]
name = "wprt-ax-driver"
version = "0.1.0"
edition = "2021"
license = "GPL-3.0-or-later"
publish = false
# Generated locally from your own disc by recompiler/dsp/build_ax.py. Never commit or upload.

[lib]
path = "ax.chunked.rs"
crate-type = ["rlib"]
''', encoding='utf-8', newline='\n')
print(f'AX driver: {len(parts)} parts, {len(e.blocks)} blocks, {len(e.locals)} locals, entry {entry}, {calls} exception checks')
