"""A second, whole-file-scoped run of instrument 3, independent of the
per-item extraction: decodes every literal in BASE's single
`rexx-bench-suite.rs` and in HEAD's two files (`rexx-bench-suite.rs` +
`rexx-bench-suite/report.rs`) combined, and multiset-diffs the two lists
(order-independent, since the split legitimately reorders content across
two files). Every literal that differs between the two sides should be
one this task's own report already names as a deliberate change; any
other difference is a finding.

Usage: whole_file_check.py <lit-decoder-binary> <base_main.rs> <head_main.rs> <head_report.rs>
"""
import collections
import subprocess
import sys


def decode(binary, path):
    proc = subprocess.run([binary, path], capture_output=True, text=True, check=True)
    return proc.stdout.splitlines()


binary, base_path, head_main_path, head_report_path = sys.argv[1:5]

base = decode(binary, base_path)
head = decode(binary, head_main_path) + decode(binary, head_report_path)

cb = collections.Counter(base)
ch = collections.Counter(head)
only_in_base = cb - ch
only_in_head = ch - cb

print(f"base total literals: {len(base)}, head total literals: {len(head)}")
print(f"literals present in base but not head: {sum(only_in_base.values())}")
for k, v in only_in_base.items():
    print(f"  x{v}  {k}")
print(f"literals present in head but not base: {sum(only_in_head.values())}")
for k, v in only_in_head.items():
    print(f"  x{v}  {k}")
