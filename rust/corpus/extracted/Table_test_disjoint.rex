/* extracted from Table::test_disjoint */
::routine main public
  emptyTable = .table~new
  table1 = .table~new
  table1[1] = "a"
  table1[2] = "b"
  table1[3] = "c"

  table2 = table1~copy
  -- same indexes, different values
  table3 = .table~new
  table1[1] = "x"
  table1[2] = "y"
  table1[3] = "z"

  -- empty tables of two different sizes...still equivalent


  self~assertTrue(emptyTable~disjoint(.table~new))
  -- empty vs. non-empty, two ways
  self~assertTrue(table1~disjoint(emptyTable))
  self~assertTrue(emptytable~disjoint(table1))
  -- simple overlap test
  self~assertFalse(table1~disjoint(table2))
  -- same indices, different values
  self~assertTrue(table1~disjoint(table3))
  -- same values, different indexes
  table3~empty
  table3[4] = "a"
  table3[5] = "b"
  table3[6] = "c"
  self~assertTrue(table1~disjoint(table3))
  -- different number of items
  table2~remove(3)
  self~assertFalse(table1~disjoint(table2))

/* add test cases for of and do over for 12.10.2014 Walter Pachl */

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
