#!/usr/bin/env python3
"""Phase 9d Task 3 Step 2, second half: send every method name each receiver's
class ancestry defines, scope by scope, through a `Message` with that scope as
its override, and record which sends end in a `native_method` refusal.

`sweep.py` sends the documented names through ordinary dispatch. A scope
override reaches what ordinary dispatch does not: an ancestor's method on a
receiver whose own class overrides or hides it (measured, oracle rc 0:
`.Message~new(.nil, .array~of('OBJECTNAME=', .Object), 'I', 'zz')~send`
answers, where `.nil~objectName = 'zz'` is 97.1).

Usage, from the repository root:

    python3 -I docs/superpowers/records/2026-10-18-phase-9d/refusals-probes/sweep2.py \
        REXX_BINARY RUN_DIR OUT_TSV

The names come from this crate's own `Class~methods(.nil)` for each class in
the receiver's ancestry, and for the oracle's from the same enumeration run on
the oracle, except for `RexxQueue`, whose construction reaches rxapi there.
Each send runs once with no argument and once with the single argument `'v'`.
"""

import concurrent.futures
import os
import re
import subprocess
import sys

ROOT = os.getcwd()
CLASS_SET = os.path.join(ROOT, "rust/corpus/docs/class-set.txt")
ORACLE = "/home/moritz/dev/repos/ooRexx/build/bin/rexx"
ORACLE_LIB = "/home/moritz/dev/repos/ooRexx/build/lib"

NATIVE_METHOD = re.compile(r'method "[^"]*" of class "[^"]*" is not implemented')

# Receivers beyond class-set.txt's: `.nil`, and the representations the seed
# sites name.
EXTRA = [
    ("nil", ".nil", "-"),
    ("ClassObject", ".Array", "-"),
    ("RexxPackage", ".Object~package", "-"),
    ("ProgramPackage", ".context~package", "-"),
    ("Context", ".context", "-"),
    ("StackFrame", ".context~stackFrames[1]", "-"),
    ("RexxInfo", ".RexxInfo", "-"),
    ("Environment", ".environment", "-"),
    ("Local", ".local", "-"),
    ("PackageMethods", ".methods", "-"),
    ("PackageRoutines", ".routines", "-"),
    ("SmallInt", "5", "-"),
    ("LongString", "'a string longer than seven bytes'", "-"),
    ("Number", "1.5", "-"),
    ("StemValue", "zzstem.", "-"),
    ("VarRef", ">zzvar", "-"),
    ("StartedMessage", ".Object~new~start('STRING')~~wait", "-"),
]


def receivers():
    found = []
    with open(CLASS_SET) as handle:
        for line in handle:
            if line.startswith("#") or not line.strip():
                continue
            cols = line.rstrip("\n").split("\t")
            if cols[6] != "-":
                found.append((cols[0], cols[6], cols[7]))
    return found + EXTRA


ENUM = """zzvar = 1
zzstem.1 = 'a'
r = %s
seen = .set~new
todo = .array~of(r~class)
do while todo~items > 0
  k = todo[1]; todo~delete(1)
  if seen~hasIndex(k) then iterate
  seen~put(k)
  s = k~methods(.nil)
  do while s~available
    say 'NAME' k~id s~index
    s~next
  end
  do c over k~superClasses; todo~append(c); end
end
exit 0
%s
"""


def run_crate(rexx, cwd, path):
    result = subprocess.run(
        ["memcap", "2G", "timeout", "-k", "2", "10", rexx, path],
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return result.returncode, result.stdout.decode("utf-8", "replace"), result.stderr.decode("utf-8", "replace")


def run_oracle(cwd, path):
    env = dict(os.environ, LD_LIBRARY_PATH=ORACLE_LIB)
    result = subprocess.run(
        ["prlimit", "--as=1073741824", "timeout", "-k", "5", "20", ORACLE, path],
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return result.returncode, result.stdout.decode("utf-8", "replace"), result.stderr.decode("utf-8", "replace")


def names(rexx, run_dir, label, expression, directive):
    path = os.path.join(run_dir, "enum_%s.rex" % label)
    with open(path, "w") as handle:
        handle.write(ENUM % (expression, "" if directive == "-" else directive))
    found = set()
    work = os.path.join(run_dir, "work")
    sides = [run_crate(rexx, work, path)]
    if label != "RexxQueue":
        sides.append(run_oracle(work, path))
    for _rc, stdout, _stderr in sides:
        for line in stdout.splitlines():
            parts = line.split(" ", 2)
            if len(parts) == 3 and parts[0] == "NAME":
                found.add((parts[1], parts[2]))
    return sorted(found)


SEND = """signal on syntax name trapped
zzvar = 1
zzstem.1 = 'a'
r = %s
m = .Message~new(r, .array~of('%s', .%s)%s)
m~send
say 'sent'
exit 0
trapped: say 'syntax' condition('O')~code
exit 0
%s
"""


def variants(label, expression, directive):
    yield "plain", expression, directive
    prefix = ".%s~new" % label
    if expression.lower().startswith(prefix.lower()):
        rest = expression[len(prefix):]
        sub = "::class ZzSub subclass %s" % label
        both = sub if directive == "-" else directive + "\n" + sub
        yield "subclass", ".ZzSub~new" + rest, both
        inner = rest[1:-1] if rest.startswith("(") and rest.endswith(")") else ""
        extra = (", " + inner) if inner else ""
        yield "enhanced", ".%s~enhanced(.table~new~~put('return 1', 'ZZPROBE')%s)" % (label, extra), directive


def main():
    rexx, run_dir, out = sys.argv[1:4]
    work = os.path.join(run_dir, "work")
    os.makedirs(work, exist_ok=True)
    probes = os.path.join(run_dir, "probes")
    os.makedirs(probes, exist_ok=True)
    jobs = []
    for label, expression, directive in receivers():
        for scope, name in names(rexx, run_dir, label, expression, directive):
            for variant, recv, extra in variants(label, expression, directive):
                for args in ("", ", 'I', 'v'"):
                    jobs.append((label, variant, scope, name, args, recv, extra))

    def one(index_job):
        index, (label, variant, scope, name, args, recv, extra) = index_job
        path = os.path.join(probes, "s%05d.rex" % index)
        quoted = name.replace("'", "''")
        with open(path, "w") as handle:
            handle.write(SEND % (recv, quoted, scope, args, "" if extra == "-" else extra))
        rc, stdout, stderr = run_crate(rexx, work, path)
        last = lambda s: s.strip().splitlines()[-1] if s.strip() else ""
        return "\t".join(
            [
                label,
                variant,
                scope,
                name,
                "1" if args else "0",
                str(rc),
                "yes" if NATIVE_METHOD.search(stderr) else "no",
                last(stdout)[:80],
                last(stderr)[:200],
                os.path.basename(path),
            ]
        )

    with open(out, "w") as sink, concurrent.futures.ThreadPoolExecutor(4) as pool:
        sink.write("receiver\tvariant\tscope\tmethod\targs\trc\tnative_method\tstdout\tstderr\tprobe\n")
        for line in pool.map(one, enumerate(jobs)):
            sink.write(line + "\n")


if __name__ == "__main__":
    main()
