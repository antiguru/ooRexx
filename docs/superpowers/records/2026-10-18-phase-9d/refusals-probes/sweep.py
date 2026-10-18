#!/usr/bin/env python3
"""Phase 9d Task 3 Step 2: send every documented method with the arguments
its syntax diagram requires, through a plain, a subclass and an enhanced
receiver, and record which sends end in the generic fallthrough's refusal.

Usage, from the repository root:

    python3 -I docs/superpowers/records/2026-10-18-phase-9d/refusals-probes/sweep.py \
        REXX_BINARY RUN_DIR OUT_TSV

RUN_DIR is a fresh empty directory the probes run in. Each probe runs under
`memcap 2G` and `timeout -k 2 10`, one process per send, because a loud
refusal ends the program and cannot be trapped.

Arguments come from the method's railroad diagram under
`oodocs/rexxref/en-US/images/classes/`: a nonterminal on the diagram's main
line is required, anything off it is optional and is not sent.
"""

import os
import re
import subprocess
import sys

ROOT = os.getcwd()
DOCS = os.path.join(ROOT, "oodocs/rexxref/en-US")
METHODS = os.path.join(ROOT, "rust/corpus/docs/class-methods.txt")
CLASS_SET = os.path.join(ROOT, "rust/corpus/docs/class-set.txt")

FALLTHROUGH = re.compile(r'method "[^"]*" of class "[^"]*" is not implemented')
LOUD = re.compile(r"is not implemented")

SVG_TEXT = re.compile(r'<text class="(terminal|nonterminal)" x="(\d+)" y="(\d+)">([^<]*)</text>')


def rows():
    with open(METHODS) as handle:
        for line in handle:
            if line.startswith("#") or not line.strip():
                continue
            cols = line.rstrip("\n").split("\t")
            yield cols


def receivers():
    found = {}
    with open(CLASS_SET) as handle:
        for line in handle:
            if line.startswith("#") or not line.strip():
                continue
            cols = line.rstrip("\n").split("\t")
            found[cols[0]] = (cols[6], cols[7])
    return found


def svg_for(section, origin):
    path = os.path.join(DOCS, origin.split(":")[0])
    with open(path) as handle:
        text = handle.read()
    at = text.find('id="%s"' % section)
    if at < 0:
        return None
    end = text.find("<section", at + 1)
    found = re.search(r'imagedata fileref="(images/classes/[^"]+\.svg)"', text[at:end if end > 0 else None])
    return os.path.join(DOCS, found.group(1)) if found else None


def required_args(svg, method):
    """The main-line nonterminals of the diagram whose first terminal names
    `method`; an empty list when the diagram has none or is not found."""
    if svg is None or not os.path.exists(svg):
        return None
    with open(svg) as handle:
        texts = SVG_TEXT.findall(handle.read())
    if not texts:
        return None
    main_y = None
    for kind, _x, y, word in texts:
        if kind == "terminal":
            main_y = y
            break
    return [word for kind, _x, y, word in texts if kind == "nonterminal" and y == main_y]


VALUES = [
    (r"^(argument|arithmetic_operator)$", "1"),
    (r"^(arguments|messageargs)$", ".array~new"),
    (r"^(first|second)$", "'v'"),
    (r"^routine$", ".Routine~new('r', 'return 1')"),
    (r"^package$", ".context~package"),
    (r"^(traceobject|othertraceobject)$", ".TraceObject~new"),
    (r"^makestringmethod$", "'STRING'"),
    (r"^stream_command$", "'query exists'"),
    (r"^(filename|path|dest)$", "'zzfile'"),
    (r"^(days|hours|minutes|microseconds|weeks|years|chars|newsize|interval)$", "1"),
    (r"^(item|value|object|obj|target|receiver|anyobject)$", "'v'"),
    (r"index|position|start|count|length|n$|^n|pos|size|number|digits|line|lines|offset|seconds|time|amount|hint|dimension|width|column|min|max", "1"),
    (r"^(class|classobj|mixinclass|superclass|metaclass)$", ".Object"),
    (r"^methods$", ".table~new"),
    (r"^method$", ".Object~method('STRING')"),
    (r"^(collection|other|array|source|indexes|items)$", ".array~of('v')"),
    (r"^(message|messagename|msg)$", "'STRING'"),
    (r"^(comparator)$", ".Comparator~new"),
    (r"^(supplier)$", ".array~of('v')~supplier"),
    (r"^(string|str|text|name|methodname|id|pattern|needle|delimiter|option|options|format|mode|command)", "'x'"),
]


def value_for(name):
    lower = name.lower()
    for pattern, value in VALUES:
        if re.search(pattern, lower):
            return value
    return "'x'"


def program(expression, directive, method, args, variant, cls, arm):
    """One send as a Rexx program."""
    lines = ["signal on syntax name trapped"]
    if arm == "class":
        recv = ".%s" % cls if variant == "plain" else ".ZzSub"
    elif variant == "plain":
        recv = expression
    elif variant == "subclass":
        prefix = ".%s~new" % cls
        if not expression.lower().startswith(prefix.lower()):
            return None
        recv = ".ZzSub~new" + expression[len(prefix):]
    else:
        prefix = ".%s~new" % cls
        if not expression.lower().startswith(prefix.lower()):
            return None
        rest = expression[len(prefix):]
        inner = rest[1:-1] if rest.startswith("(") and rest.endswith(")") else ""
        extra = (", " + inner) if inner else ""
        recv = ".%s~enhanced(.table~new~~put('return 1', 'ZZPROBE')%s)" % (cls, extra)
    lines.append("r = " + recv)
    quoted = method.replace('"', '""')
    lines.append('r~"%s"(%s)' % (quoted, ", ".join(args)))
    lines.append("say 'sent'")
    lines.append("exit 0")
    lines.append("trapped: say 'syntax' condition('O')~code")
    lines.append("exit 0")
    if variant != "plain" or arm == "class":
        lines.append("::class ZzSub subclass %s" % cls)
    if directive != "-":
        lines.append(directive)
    return "\n".join(lines) + "\n"


def run(rexx, run_dir, path):
    result = subprocess.run(
        ["memcap", "2G", "timeout", "-k", "2", "10", rexx, path],
        cwd=run_dir,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return result.returncode, result.stdout.decode("utf-8", "replace"), result.stderr.decode("utf-8", "replace")


def main():
    rexx, run_dir, out = sys.argv[1:4]
    have = receivers()
    probes = os.path.join(run_dir, "probes")
    os.makedirs(probes, exist_ok=True)
    work = os.path.join(run_dir, "work")
    os.makedirs(work, exist_ok=True)
    with open(out, "w") as sink:
        sink.write("class\tmethod\tarm\tvariant\targs\trc\tfallthrough\tloud\tstdout\tstderr\n")
        for count, cols in enumerate(rows()):
            cls, method, arm, status, section, origin = cols[:6]
            if cls not in have:
                continue
            expression, directive = have[cls]
            if arm == "instance" and expression == "-":
                continue
            names = required_args(svg_for(section, origin), method)
            args = [value_for(name) for name in (names or [])]
            for variant in ("plain", "subclass", "enhanced"):
                if arm == "class" and variant == "enhanced":
                    continue
                text = program(expression, directive, method, args, variant, cls, arm)
                if text is None:
                    continue
                path = os.path.join(probes, "p%04d_%s.rex" % (count, variant))
                with open(path, "w") as handle:
                    handle.write(text)
                rc, stdout, stderr = run(rexx, work, path)
                last = lambda s: s.strip().splitlines()[-1] if s.strip() else ""
                sink.write(
                    "\t".join(
                        [
                            cls,
                            method,
                            arm,
                            variant,
                            ",".join(args),
                            str(rc),
                            "yes" if FALLTHROUGH.search(stderr) else "no",
                            "yes" if LOUD.search(stderr) else "no",
                            last(stdout)[:120],
                            last(stderr)[:200],
                        ]
                    )
                    + "\n"
                )
                sink.flush()


if __name__ == "__main__":
    main()
