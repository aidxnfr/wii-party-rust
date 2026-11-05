# Finds C++ expressions whose meaning would change under Rust precedence: a bit operator (& ^ |) and a
# comparison (== != < > <= >=) at the same parenthesis depth, or C-style casts / ternaries.
import re, sys
tok = re.compile(r'&&|\|\||==|!=|<=|>=|<<|>>|[()&|^<>?]')
bad = 0; seen = set()
for p in sys.argv[1:]:
    for n, line in enumerate(open(p, errors='replace'), 1):
        s = line.strip()
        if s.startswith(('void ', '#', '//', 'case ', 'static const', 'std::')) or 'template' in s: continue
        s = re.sub(r'static_cast<\w+>', 'F', s); s = re.sub(r'wp::\w+<[^>]*>', 'F', s)
        levels = [set()]
        for t in tok.findall(s):
            if t == '(': levels.append(set())
            elif t == ')':
                if len(levels) > 1: levels.pop()
            elif t in ('&', '|', '^'): levels[-1].add('bit')
            elif t in ('==', '!=', '<=', '>=', '<', '>'): levels[-1].add('cmp')
            elif t == '?': levels[-1].add('tern')
            if {'bit', 'cmp'} <= levels[-1] or 'tern' in levels[-1]:
                key = re.sub(r'0x[0-9a-f]+u?|\d+', 'N', s)
                if key not in seen and len(seen) < 40: print(f'{p}:{n}: {s[:140]}')
                seen.add(key); bad += 1; break
print('suspicious lines:', bad, 'distinct shapes:', len(seen))
