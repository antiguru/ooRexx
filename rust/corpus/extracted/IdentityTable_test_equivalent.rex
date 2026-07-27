/* extracted from IdentityTable::test_equivalent */
::routine main public
  emptytable = .identitytable~new
  table1 = .identitytable~new
  table1[1] = "a"
  table1[2] = "b"
  table1[3] = "c"

  table2 = table1~copy

  -- empty tables of two different sizes...still equivalent


  self~assertTrue(emptytable~equivalent(.identitytable~new))
  -- empty vs. non-empty, two ways
  self~assertFalse(table1~equivalent(emptytable))
  self~assertFalse(emptytable~equivalent(table1))
  -- simple true test
  self~assertTrue(table1~equivalent(table2))
  table2[2] = 'z'
  -- same number of items, different values
  self~assertFalse(table1~equivalent(table2))
  -- same values, different indexes
  table3 = table1~copy
  table3~remove(2)
  table3[4] = "b"
  self~assertFalse(table1~equivalent(table3))
  -- different number of items
  table3~remove(4)
  self~assertFalse(table1~equivalent(table3))
  self~assertFalse(table3~equivalent(table1))

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
