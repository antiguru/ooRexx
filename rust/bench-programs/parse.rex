/* PARSE, the shapes rexxcps uses: a positional pattern, a variable pattern,
   and PARSE VAR with several targets. exec_parse is 6.8% of rexxcps' profile
   and no bench program exercises it at all. */
p0 = 'b'
total = 0
do i = 1 to 200000
  parse value 'Foo Bar' with v1 +5 v2 .
  rc = 'This is an awfully boring program'; parse var rc p1 (p0) p5
  rc = 'is an awfully boring program This'; parse var rc p2 (p0) p6
  parse value 'alpha beta gamma delta' with a1 a2 a3 a4
  total = total + length(v1) + length(p1) + length(a3)
end
say total
