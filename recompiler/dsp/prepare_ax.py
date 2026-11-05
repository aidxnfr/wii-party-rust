# Front end for the upstream-generated AX microcode function (ax.cpp): adapts C++ spelling to parseable C
# without omitting any statement. Inputs come from the environment set by recompiler/pipeline.py:
#   WP_UPSTREAM_DIR  pinned upstream checkout (headers: engine/include, third_party/dolphin)
#   WP_AX_CPP        the generated ax.cpp
from pathlib import Path
import os, re
from pycparser import c_parser, c_ast  # noqa: F401  (re-exported for emit_ax.py)

HERE = Path(__file__).resolve().parent
ROOT = Path(os.environ["WP_UPSTREAM_DIR"])


def prepare():
    source = Path(os.environ["WP_AX_CPP"]).read_text(encoding="utf-8")
    start = source.index('int run_ax('); end = source.rfind('\n}'); text = source[start:end].strip()
    text = text.replace('int run_ax(DSP::Interpreter::Interpreter& in, DSP::SDSP& s, int cycles, bool& idle)', 'int run_ax(Interpreter* in, SDSP* s, int cycles, bool* idle)')
    # Native SDK methods remain explicit calls; they are not translated to guessed implementations.
    text = text.replace('s.GetAnalyzer().IsLoopEnd(', 'sdk_IsLoopEnd(s, ').replace('s.GetAnalyzer().IsIdleSkip(', 'sdk_IsIdleSkip(s, ')
    text = re.sub(r'\bs\.(\w+)\(', lambda m: 'sdk_' + m[1] + '(s, ', text)
    text = re.sub(r'\bin\.(\w+)\(', lambda m: 'sdk_' + m[1] + '(in, ', text)
    text = text.replace('(s, )', '(s)').replace('(in, )', '(in)')
    text = re.sub(r'static_cast<([^>]+)>', lambda m: '(' + ('int' if m[1] == 'DSP::StackRegister' else m[1]) + ')', text)
    text = text.replace('s.', 's->').replace('idle = true', '*idle = true')
    # Scope constants are left as distinct identifiers; types/values are resolved from SDK headers.
    text = text.replace('::', '_')
    prefix = ('typedef signed char s8;typedef unsigned char u8;typedef short s16;typedef unsigned short u16;typedef int s32;'
              'typedef unsigned int u32;typedef long long s64;typedef unsigned long long u64;typedef _Bool bool;'
              'typedef struct SDSP SDSP;typedef struct Interpreter Interpreter;\n')
    return prefix + text + '\n'
