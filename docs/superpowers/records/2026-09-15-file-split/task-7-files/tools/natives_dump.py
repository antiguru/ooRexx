"""The native table instrument: what `ObjectModel::build` files under each
`MethodId`, compared by function name rather than by which table row put it
there, so it sees a reordered chain that changes a collision's winner.

  natives_dump.py patch DISPATCH_RS   insert a temporary dump into `build`
                                      (in a scratch worktree, never committed)
  natives_dump.py resolve BIN STDERR  map the dumped addresses to function
                                      names with `nm`, one line per MethodId

The dump prints `put_native`'s runtime address as an anchor, so a PIE load
offset cancels: symbol = nm(address - anchor + nm(put_native)).
"""
import subprocess
import sys

ANCHOR_TEXT = '        let string = classes.lookup("String").expect("String is a native class");\n'
DUMP = """        eprintln!("ANCHOR {:x}", put_native as usize);
        for (index, entry) in natives.iter().enumerate() {
            if let Some(entry) = entry {
                let arity = match entry.arity {
                    Arity::Fixed(count) => count as isize,
                    Arity::Counted => -1,
                };
                eprintln!("NATIVE {index} {arity} {:x}", entry.run as usize);
            }
        }
"""

if sys.argv[1] == "patch":
    path = sys.argv[2]
    text = open(path).read()
    assert text.count(ANCHOR_TEXT) == 1
    open(path, "w").write(text.replace(ANCHOR_TEXT, DUMP + ANCHOR_TEXT))
elif sys.argv[1] == "resolve":
    binary, stderr = sys.argv[2], sys.argv[3]
    symbols = {}
    anchor_static = None
    for line in subprocess.run(["nm", "-C", binary], capture_output=True, text=True, check=True).stdout.splitlines():
        parts = line.split(" ", 2)
        if line.startswith(" ") or len(parts) != 3 or parts[1] not in ("t", "T"):
            continue
        address = int(parts[0], 16)
        name = parts[2]
        symbols.setdefault(address, []).append(name)
        if name.endswith("rexx_exec::dispatch::put_native"):
            assert anchor_static is None
            anchor_static = address
    assert anchor_static is not None
    anchor = None
    rows = []
    for line in open(stderr):
        if line.startswith("ANCHOR "):
            assert anchor is None, "one build per run"
            anchor = int(line.split()[1], 16)
        elif line.startswith("NATIVE "):
            _, index, arity, address = line.split()
            names = symbols[int(address, 16) - anchor + anchor_static]
            short = sorted(set(n.split("::")[-1] for n in names))
            rows.append(f"{index} {arity} {'|'.join(short)}")
    for row in rows:
        print(row)
