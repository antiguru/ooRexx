import sys
kind, n, k = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
pre = f"n = {n}\ny = 'abc'\nx = ''\n"
body = "  nop\n" if kind == 'nop' else "  x = y\n"
sys.stdout.write(pre + "do i = 1 to n\n" + body * k + "end\nsay x\n")
