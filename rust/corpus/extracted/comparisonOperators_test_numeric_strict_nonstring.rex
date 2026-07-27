/* extracted from comparisonOperators::test_numeric_strict_nonstring */
::routine main public
  signal off nostring
  do op1 over 123, 1/3, 1e3, '123', 'a'
    op2 = .Object~new
    -- op1 <=> an Object
    self~assertTrue(op1 \== op2)
    self~assertTrue(op1 <> op2)
    self~assertTrue(op1 >< op2)
    self~assertTrue(op1 < op2)
    self~assertTrue(op1 \= op2)
    self~assertTrue(op1 <= op2)
    self~assertTrue(op1 \> op2)
    self~assertTrue(op1 << op2)
    self~assertTrue(op1 <<= op2)
    self~assertTrue(op1 \>> op2)
    self~assertFalse(op1 = op2)
    self~assertFalse(op1 == op2)
    self~assertFalse(op1 > op2)
    self~assertFalse(op1 >= op2)
    self~assertFalse(op1 \< op2)
    self~assertFalse(op1 >> op2)
    self~assertFalse(op1 >>= op2)
    self~assertFalse(op1 \<< op2)
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
