/* extracted from Array::test_empty_then_append */
::routine main public
  e = .array~new
  do i = 1 to 10000
    e~append(i)
  end
  self~assertSame(10000, e~items, 'array e must have 10,000 items')
  e~empty
  self~assertSame(0, e~items, 'array e must have 0 items after empty')

  do i = 1 to 10000
    e~append(.object~new)
  end
  self~assertSame(10000, e~items, 'array e must have 10,000 items after empty then append')

  do i = 1 to 10000 by 4
    e~remove(i)
  end
  self~assertSame(7500, e~items, 'array e must have 7500 itmes after removing 25%')

  f = e~makearray
  self~assertSame(7500, f~items, 'array f must have 7500 itmes after makearray')
  do i = 1 to f~items
    self~assertSame(.object, f~remove(i)~class, 'each item must be remoable and an instance of .Object')
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
