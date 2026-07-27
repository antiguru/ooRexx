parse arg infile
do while lines(infile) > 0
  line = linein(infile)
  parse var line d "|" a "|" op "|" b
  say line || "=" || calc(d, a, op, b)
end
exit 0
calc:
  parse arg dd, aa, oo, bb
  signal on syntax name oops
  numeric digits dd
  interpret "r = aa" oo "bb"
  return r
oops:
  return "<E" || rc || ">"
