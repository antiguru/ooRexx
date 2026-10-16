import sys, os
d = sys.argv[1]
progs = {
 "prefix_after_op": b"x = 1 + -\n",
 "say_prefix": b"say -\n",
 "not_prefix": b"say \\\n",
 "hex_bad_group": b"x = '12 3'x\n",
 "bin_bad_group": b"x = '0101 1'b\n",
 "ref_amp_literal": b"x = >'&1'\n",
 "ref_raw_byte": b"x = >'80'x\n",
 "bad_emoji": b"x = a \xf0\x9f\x98\x80 b\n",
 "bad_truncated": b"x = a \xc3",
 "bad_overlong": b"x = a \xc0\xaf b\n",
 "resource_amp": b"::resource '&1' end '&2'\nline\n",
 "comment_line3": b"say 1\n\nsay 2 /* open /* nested */\nmore\n",
 "multi_line_bracket": b"x = a[1,,\n  [2\n",
}
for name, body in progs.items():
    with open(os.path.join(d, name + ".rex"), "wb") as f:
        f.write(body)
