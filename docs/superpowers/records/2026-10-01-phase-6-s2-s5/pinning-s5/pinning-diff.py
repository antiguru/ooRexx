#!/usr/bin/env python3
"""Compares a pinning.log against phase-6-pinning.md's S3 close.

Usage: python3 pinning-diff.py PINNING_LOG PINNING_MD
Prints the arrivals per park of each mode in the log, then a unified diff
of each mode's waits-by-kind table against the S3 close's; no diff lines
means the tables are identical.
"""
import collections, difflib, sys

log = open(sys.argv[1]).read().split('\n')
md = open(sys.argv[2]).read().split('\n')


def tables(lines, header):
    out = []
    for at, line in enumerate(lines):
        if line.endswith(header):
            rows = [header]
            for row in lines[at + 1:]:
                if not row.startswith('|'):
                    break
                rows.append(row)
            out.append(rows)
    return out


s3 = md[md.index('## S3 close'):md.index('## S4 close')]
s3_waits = tables(s3, '| wait | park | frames | count |')
log_waits = tables(log, '| wait | park | frames | count |')
log_parks = tables(log, '| park | frames | tests | arrivals |')
assert len(s3_waits) == 2 and len(log_waits) == 2 and len(log_parks) == 2
for mode, parks, waits, ours in zip(['normal', 'every opportunity'], log_parks, s3_waits, log_waits):
    arrivals = collections.Counter()
    for row in parks[2:]:
        cells = [c.strip() for c in row.strip('|').split('|')]
        arrivals[cells[0]] += int(cells[3])
    print(mode, 'arrivals:', ', '.join(f'{k} {v}' for k, v in arrivals.items()))
    for line in difflib.unified_diff(waits, ours, 'S3 close', 'log', lineterm=''):
        print(line)
    print(mode, 'waits-by-kind diff end')
