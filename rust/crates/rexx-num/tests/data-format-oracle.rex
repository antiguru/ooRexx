/* Independent oracle driver for FORMAT and TRUNC.
   Case format:  digits|func|number|a1|a2|a3|a4
   An empty argument field means "omitted". Prints case=result, with
   <Enn> for a trapped syntax error. */
parse arg infile
do while lines(infile) > 0
  line = linein(infile)
  say line || "=" || run(line)
end
exit 0

run:
  parse arg spec
  parse var spec d "|" fn "|" num "|" a1 "|" a2 "|" a3 "|" a4
  signal on syntax name oops
  numeric digits d
  call = fn || "(" || quoted(num)
  if a1 \== "" | a2 \== "" | a3 \== "" | a4 \== "" then call = call || "," || a1
  if a2 \== "" | a3 \== "" | a4 \== "" then call = call || "," || a2
  if a3 \== "" | a4 \== "" then call = call || "," || a3
  if a4 \== "" then call = call || "," || a4
  call = call || ")"
  interpret "r =" call
  return r
oops:
  return "<E" || rc || ">"

quoted:
  parse arg v
  return "'" || v || "'"
