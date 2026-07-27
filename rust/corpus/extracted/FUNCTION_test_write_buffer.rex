/* extracted from FUNCTION::test_write_buffer */
::routine main public

  array = .array~new
  crlf = '0d0a'x
  lf = '0a'x
  cr = '0d'x

  -- one string, no linends
  address io 'BUFFEROUTPUT' with input using "This is a test" output using (array)
  self~assertSame(1, array~items)
  self~assertSame("This is a test", array[1])

  -- one string, crlf linend
  address io 'BUFFEROUTPUT' with input using ("Line1"||crlf) output using (array)
  self~assertSame(1, array~items)
  self~assertSame("Line1", array[1])

  -- one string, lf linend
  address io 'BUFFEROUTPUT' with input using ("Line2"||lf) output using (array)
  self~assertSame(1, array~items)
  self~assertSame("Line2", array[1])

  -- two strings, no linefeeds
  address io 'BUFFEROUTPUT' with input using (("Line1", "Line2")) output using (array)
  self~assertSame(1, array~items)
  self~assertSame("Line1Line2", array[1])

  -- three strings, no linefeeds
  address io 'BUFFEROUTPUT' with input using (("Line1", "Line2", "Line3")) output using (array)
  self~assertSame(1, array~items)
  self~assertSame("Line1Line2Line3", array[1])

  -- one string, crlf linend in the middle
  address io 'BUFFEROUTPUT' with input using ("Line1"||crlf||"Line2") output using (array)
  self~assertSame(2, array~items)
  self~assertSame("Line1", array[1])
  self~assertSame("Line2", array[2])

  -- one string, lf linend in the middle
  address io 'BUFFEROUTPUT' with input using ("Line3"||lf||"Line4") output using (array)
  self~assertSame(2, array~items)
  self~assertSame("Line3", array[1])
  self~assertSame("Line4", array[2])

  -- two strings, crlf linened split across two buffers
  address io 'BUFFEROUTPUT' with input using (("Line1"||cr, lf||"Line2")) output using (array)
  self~assertSame(2, array~items)
  self~assertSame("Line1", array[1])
  self~assertSame("Line2", array[2])

  -- one string, naked cr  in the middle
  address io 'BUFFEROUTPUT' with input using ("Line3"||cr||"Line4") output using (array)
  self~assertSame(1, array~items)
  self~assertSame("Line3"||cr||"Line4", array[1])

  -- two strings, naked crlf at end if first buffter
  address io 'BUFFEROUTPUT' with input using (("Line1"||cr, "Line2")) output using (array)
  self~assertSame(1, array~items)
  self~assertSame("Line1"||cr||"Line2", array[1])

  -- two strings, multiple line feeds, no line feed at buffer boundary
  address io 'BUFFEROUTPUT' with input using (("Line1"||crlf||"Line2"||crlf||"Li", "ne3"||crlf||"Line4")) output using (array)
  self~assertSame(4, array~items)
  self~assertSame("Line1", array[1])
  self~assertSame("Line2", array[2])
  self~assertSame("Line3", array[3])
  self~assertSame("Line4", array[4])

  -- two strings, multiple line feeds, no line feed at buffer boundary
  address io 'BUFFEROUTPUT' with input using (("Line5"||lf||"Line6"||lf||"Li", "ne7"||lf||"Line8")) output using (array)
  self~assertSame(4, array~items)
  self~assertSame("Line5", array[1])
  self~assertSame("Line6", array[2])
  self~assertSame("Line7", array[3])
  self~assertSame("Line8", array[4])

  -- A number of null line variants
  address io 'BUFFEROUTPUT' with input using ((crlf"Line1"||crlf||crlf||"Line2"||cr, lf||"Line3")) output using (array)
  self~assertSame(5, array~items)
  self~assertSame("", array[1])
  self~assertSame("Line1", array[2])
  self~assertSame("", array[3])
  self~assertSame("Line2", array[4])
  self~assertSame("Line3", array[5])


  -- one string, no linends
  address io 'BUFFERERROR' with input using "This is a test" error using (array)
  self~assertSame(1, array~items)
  self~assertSame("This is a test", array[1])

  -- one string, crlf linend
  address io 'BUFFERERROR' with input using ("Line1"||crlf) error using (array)
  self~assertSame(1, array~items)
  self~assertSame("Line1", array[1])

  -- one string, lf linend
  address io 'BUFFERERROR' with input using ("Line2"||lf) error using (array)
  self~assertSame(1, array~items)
  self~assertSame("Line2", array[1])

  -- two strings, no linefeeds
  address io 'BUFFERERROR' with input using (("Line1", "Line2")) error using (array)
  self~assertSame(1, array~items)
  self~assertSame("Line1Line2", array[1])

  -- three strings, no linefeeds
  address io 'BUFFERERROR' with input using (("Line1", "Line2", "Line3")) error using (array)
  self~assertSame(1, array~items)
  self~assertSame("Line1Line2Line3", array[1])

  -- one string, crlf linend in the middle
  address io 'BUFFERERROR' with input using ("Line1"||crlf||"Line2") error using (array)
  self~assertSame(2, array~items)
  self~assertSame("Line1", array[1])
  self~assertSame("Line2", array[2])

  -- one string, lf linend in the middle
  address io 'BUFFERERROR' with input using ("Line3"||lf||"Line4") error using (array)
  self~assertSame(2, array~items)
  self~assertSame("Line3", array[1])
  self~assertSame("Line4", array[2])

  -- two strings, crlf linened split across two buffers
  address io 'BUFFERERROR' with input using (("Line1"||cr, lf||"Line2")) error using (array)
  self~assertSame(2, array~items)
  self~assertSame("Line1", array[1])
  self~assertSame("Line2", array[2])

  -- one string, naked cr  in the middle
  address io 'BUFFERERROR' with input using ("Line3"||cr||"Line4") error using (array)
  self~assertSame(1, array~items)
  self~assertSame("Line3"||cr||"Line4", array[1])

  -- two strings, naked crlf at end if first buffter
  address io 'BUFFERERROR' with input using (("Line1"||cr, "Line2")) error using (array)
  self~assertSame(1, array~items)
  self~assertSame("Line1"||cr||"Line2", array[1])

  -- two strings, multiple line feeds, no line feed at buffer boundary
  address io 'BUFFERERROR' with input using (("Line1"||crlf||"Line2"||crlf||"Li", "ne3"||crlf||"Line4")) error using (array)
  self~assertSame(4, array~items)
  self~assertSame("Line1", array[1])
  self~assertSame("Line2", array[2])
  self~assertSame("Line3", array[3])
  self~assertSame("Line4", array[4])

  -- two strings, multiple line feeds, no line feed at buffer boundary
  address io 'BUFFERERROR' with input using (("Line5"||lf||"Line6"||lf||"Li", "ne7"||crlf||"Line8")) error using (array)
  self~assertSame(4, array~items)
  self~assertSame("Line5", array[1])
  self~assertSame("Line6", array[2])
  self~assertSame("Line7", array[3])
  self~assertSame("Line8", array[4])


::class shim public
::method assertEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotEquals
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method assertTrue
  use arg condition
  if \condition then do
    say "FAIL expected true actual["condition"]"
    exit 1
  end
::method assertFalse
  use arg condition
  if condition then do
    say "FAIL expected false actual["condition"]"
    exit 1
  end
::method assertNull
  use arg actual
  if actual \== .nil then do
    say "FAIL expected nil actual["actual"]"
    exit 1
  end
::method assertNotNull
  use arg actual
  if actual == .nil then do
    say "FAIL expected non-nil actual nil"
    exit 1
  end
::method assertSame
  use arg expected, actual
  if \(expected == actual) then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotSame
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method expectSyntax
  use arg code
  nop
::method assertListEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertArrayEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
