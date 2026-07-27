/* extracted from Bag::test_equivalent */
::routine main public
  -- empty collection
  self~assertTrue(.bag~new~equivalent(.bag~new))
  -- empty vs. non-empty, two ways
  self~assertFalse(.bag~of(1,2,3)~equivalent(.bag~new))
  self~assertFalse(.bag~new~equivalent(.bag~of(1,2,3)))
  -- simple true test
  self~assertTrue(.bag~of(1,2,3)~equivalent(.bag~of(1,2,3)))
  -- same number of items, different values
  self~assertFalse(.bag~of(1,2,3)~equivalent(.bag~of(1,2,4)))
  -- duplicate items vs. non-duplicate items
  self~assertFalse(.bag~of(1,2,3)~equivalent(.bag~of(1,2,2,3)))
  self~assertFalse(.bag~of(1,2,2,3)~equivalent(.bag~of(1,2,3)))
  -- mismatch on number of arguments, tested both ways
  self~assertFalse(.bag~of(1,2,3)~equivalent(.bag~of(1,2,3,4)))
  self~assertFalse(.bag~of(1,2,3,4)~equivalent(.bag~of(1,2,3)))

  -- simple cross class test using an array.  The array is converted to a set using the items
  self~assertTrue(.set~of(1,2,3)~equivalent(.array~of(1,2,3)))
  -- and the same using a sparse array
  self~assertTrue(.set~of(1,2,3)~equivalent(.array~of(1,,2,,3)))

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
