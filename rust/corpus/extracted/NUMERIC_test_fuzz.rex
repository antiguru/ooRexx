/* extracted from NUMERIC::test_fuzz */
::routine main public
  numeric fuzz 0
  a = 123456789e2
  b = 123456785e2
  do 21
    a /= 10; b /= 10
    self~assertFalse(a  = b, "fuzz 0" a "=" b)
    self~assertFalse(a <= b, "fuzz 0" a "<=" b)
    self~assertTrue( a >= b, "fuzz 0" a "<=" b)
    self~assertTrue( a \= b, "fuzz 0" a "\=" b)
    self~assertFalse(a  < b, "fuzz 0" a "<" b)
    self~assertTrue( a  > b, "fuzz 0" a ">" b)
  end
  do f = 1 to 2
    numeric fuzz f
    a = 123456789e2
    b = 123456785e2
    do 21
      a /= 10; b /= 10
      self~assertTrue( a  = b, "fuzz" f a "=" b)
      self~assertTrue( a <= b, "fuzz" f a "<=" b)
      self~assertTrue( a >= b, "fuzz" f a "<=" b)
      self~assertFalse(a \= b, "fuzz" f a "\=" b)
      self~assertFalse(a  < b, "fuzz" f a "<" b)
      self~assertFalse(a  > b, "fuzz" f a ">" b)
    end
  end

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
