#!/usr/bin/env python3
# The functions of tagged-sites.txt's non-test rows that iterate anything
# (`.iter()`, `.values()`, `.keys()`, `.iter_mut()`, `.values_mut()` or
# `.drain()` with a `for`, `.map(` or `.filter`): the candidates for a walk of
# the interpreter's own tables. Run from rust/ after tagged-sites.py, with its
# output as the argument.
import re, sys
fns = {}
for line in open(sys.argv[1]):
    loc, fn, test = line.rstrip('\n').split('\t')
    if test:
        continue
    path, number = loc.rsplit(':', 1)
    fns.setdefault((path, fn), []).append(int(number))
for (path, fn), numbers in sorted(fns.items()):
    src = open(path).read().split('\n')
    start = max(i for i in range(numbers[0]) if re.search(r'\bfn\s+' + fn + r'\b', src[i]))
    indent = len(src[start]) - len(src[start].lstrip())
    end = start + 1
    while end < len(src) and not (src[end].startswith(' ' * indent + '}')
                                  and len(src[end]) - len(src[end].lstrip()) == indent):
        end += 1
    body = '\n'.join(src[start:end])
    if re.search(r'\.(iter|values|keys|iter_mut|values_mut|drain)\(\)', body) \
            and re.search(r'for |\.map\(|\.filter', body):
        print(f"{path}:{start + 1}\t{fn}")
