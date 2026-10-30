import pathlib, json, sys, hashlib, difflib
name, direction = sys.argv[1:]
p, old, new = json.loads(pathlib.Path(__file__).with_name('mutation-edits.json').read_text())[name]
p = pathlib.Path(p)
s = p.read_text()
if direction == 'restore': old, new = new, old
assert old and s.count(old) == 1, (name, direction, s.count(old))
t = s.replace(old, new, 1)
record = pathlib.Path('docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments')
(record / (name + '-' + direction + '.diff')).write_text(''.join(difflib.unified_diff(s.splitlines(True), t.splitlines(True), fromfile=str(p), tofile=str(p))))
p.write_text(t)
print(name, direction, hashlib.sha256(s.encode()).hexdigest(), hashlib.sha256(t.encode()).hexdigest())
