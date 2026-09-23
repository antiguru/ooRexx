import sys
kind, n = sys.argv[1], int(sys.argv[2])
if kind == 'nop':
    head = """/* The per-clause floor: a counted loop whose body is 100 `nop` clauses,
   one per line, so the loop step is amortised over the clauses and the
   measurement is the fixed cost every clause pays. */
"""
    pre = f"n = {n}\n"
    body = "  nop\n"
    tail = "say 'done'\n"
else:
    head = """/* The simplest assignment: the `nop.rex` shape with 100 `x = y` clauses,
   `y` assigned once before the loop, so the measurement is the per-clause
   cost plus one variable read and one variable write. */
"""
    pre = f"n = {n}\ny = 'abc'\n"
    body = "  x = y\n"
    tail = "say x\n"
sys.stdout.write(head + pre + "do i = 1 to n\n" + body * 100 + "end\n" + tail)
