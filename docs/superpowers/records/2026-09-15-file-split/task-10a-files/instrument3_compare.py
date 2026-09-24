"""Instrument 3, run for real: for every moved item, decode its literals
(base vs head) with the token-stream-walking Rust decoder
(instrument3_lit_decoder), cross-check the decoder's own decodable-literal
count against an independently written lexer-level count
(instrument3_independent_lexer_count.py) on *both* sides, and diff the
decoded literal sequences base vs head.
"""
import subprocess
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from instrument3_independent_lexer_count import count_string_char_literals

DECODER = sys.argv[1]  # path to the lit-decoder release binary
ITEMS_DIR = sys.argv[2]  # directory with base/<name>.rs and head/<name>.rs

names = sorted(
    f[:-3] for f in os.listdir(os.path.join(ITEMS_DIR, "base")) if f.endswith(".rs")
)

decode_mismatches = 0
control_mismatches = 0
total_decodable = 0

for name in names:
    row = {}
    for side in ("base", "head"):
        path = os.path.join(ITEMS_DIR, side, f"{name}.rs")
        proc = subprocess.run([DECODER, path], capture_output=True, text=True)
        if proc.returncode != 0:
            print(f"PARSE-ERROR {name} ({side}): {proc.stderr}")
            control_mismatches += 1
            continue
        decoded_lines = proc.stdout.splitlines()
        decodable_count = None
        for line in proc.stderr.splitlines():
            if line.startswith("decodable_count="):
                decodable_count = int(line.split("=", 1)[1])
        with open(path) as f:
            text = f.read()
        independent_count = count_string_char_literals(text)
        row[side] = (decoded_lines, decodable_count, independent_count)

    base_lines, base_decoder_count, base_lexer_count = row["base"]
    head_lines, head_decoder_count, head_lexer_count = row["head"]

    control_ok = (base_decoder_count == base_lexer_count) and (
        head_decoder_count == head_lexer_count
    )
    if not control_ok:
        control_mismatches += 1

    decode_ok = base_lines == head_lines
    if not decode_ok:
        decode_mismatches += 1

    total_decodable += base_decoder_count

    status = "OK" if (decode_ok and control_ok) else "FAIL"
    print(
        f"{status}  {name}: base/head decoder-vs-lexer counts "
        f"base={base_decoder_count}/{base_lexer_count} "
        f"head={head_decoder_count}/{head_lexer_count}  "
        f"({len(base_lines)} total literal tokens, {base_decoder_count} decodable) "
        f"{'decode-match' if decode_ok else 'DECODE-DIFFERS'}"
    )
    if not decode_ok:
        import difflib

        for line in difflib.unified_diff(base_lines, head_lines, "base", "head", lineterm=""):
            print("    " + line)

print()
print(f"{len(names)} items compared, {total_decodable} decodable literals (STR/BYTESTR/CHAR/BYTE) total.")
print(f"decode mismatches: {decode_mismatches}")
print(f"control (decoder-count vs independent-lexer-count) mismatches: {control_mismatches}")
