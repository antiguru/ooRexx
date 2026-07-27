parse arg infile
numeric digits 9
do while lines(infile) > 0
  v = linein(infile)
  say v || "|" || canon(v)
end
exit 0

canon:
  parse arg s
  signal on syntax name oops
  return s + 0
oops:
  return "<SYNTAX>"
