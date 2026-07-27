/* extracted from json::test_number */
::routine main public
  -- strictly speaking, the JSON number grammar doesn't allow numbers like
  -- 0123, 1., +3, or .2
  -- json.cls allows for all valid Rexx numbers
  -- https://www.ecma-international.org/publications/files/ECMA-ST/ECMA-404.pdf
  -- number ::= '-'? ('0' | [1-9] [0-9]+) ('.' [0-9]+)? (('e' | 'E') ( | '+' | '-') [0-9]+)?

  j = .Json~new
  -- integer
  do number over 0, 1, -4, 123, 12345678901234567890
    self~assertSame(number, j~toJson(j~fromJson(number)))
  end

  -- fractions
  do number over 1.2, -0.003, 0.00000000000000
    self~assertSame(number, j~toJson(j~fromJson(number)))
  end

  -- exponential
  do number over "1e0", "-1E0", 1E999999999, -2e5, 1e-9, -9e-9, 0.00003e4, -1.23e0
    self~assertSame(number, j~toJson(j~fromJson(number)))
  end

  -- whitespace
  number = 123
  self~assertSame(number, j~toJson(j~fromJson(" " number)))
  self~assertSame(number, j~toJson(j~fromJson(number || '0d 0a'x)))
  self~assertSame(number, j~toJson(j~fromJson('20 0d 09 0a'x || number || '0a 09 20 0a'x)))


-- string

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
