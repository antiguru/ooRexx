/* An input stream object with no STATE method, the FUNCTION group's
   ArrayInputStream shape, through orxfunction's I/O handler and a shell. */
call TestAddCommandEnvironment "io", "redirecting"
out = .array~new
address io 'INPUTOUTPUT' with input using (.ArrayInputStream~new(("Line1", "Line2"))) output using (out)
say 'handler' rc out~items
address system 'cat' with input using (.ArrayInputStream~new(("Line1", "Line2"))) output using (out)
say 'shell' rc out~items
::routine TestAddCommandEnvironment PUBLIC EXTERNAL "LIBRARY orxfunction"
::class ArrayInputStream public inherit InputStream
::method init
  expose lines current
  use strict arg lines
  current = 1
::method linein
  expose lines current
  if current > lines~items then raise notready additional(self) return("")
  current += 1
  return lines[current - 1]
::method lines
  expose lines current
  return max(0, lines~items - current + 1)
