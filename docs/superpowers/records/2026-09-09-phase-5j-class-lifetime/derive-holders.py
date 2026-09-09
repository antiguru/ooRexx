import re, sys
src = open('crates/rexx-exec/src/lib.rs').read()
# the struct's field declarations
start = src.index('struct Interp {'); end = src.index('\n}\n', start)
decls = re.findall(r'^    ([a-z_]+): (.+),$', src[start:end], re.M)
# object_roots' destructure: a field bound `_` is not handed to the collector
i = src.index('let Interp {'); j = src.index('} = self;', i)
bound = re.findall(r'^            ([a-z_]+)(: _)?,$', src[i:j], re.M)
rooted = {name for name, underscore in bound if not underscore}
print('field\tholds_objref\thanded_to_collector')
for name, ty in decls:
    print(f'{name}\t{"yes" if "ObjRef" in ty else "no"}\t{"yes" if name in rooted else "no"}')
