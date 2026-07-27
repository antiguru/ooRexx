/* extracted from FUNCTION::test_rexxqueue */
::routine main public

  .rexxqueue~create('ADDRESSWITH')
  queue = .rexxqueue~new('ADDRESSWITH')

  address io 'INPUTOUTPUT' with input using 'This is a test' output using(queue)
  self~assertSame(1, queue~queued)
  self~assertSame("This is a test", queue~pull)

  queue~queue('Line1')
  array = .array~new

  address io 'INPUTOUTPUT' with input using(queue) output using(array)
  self~assertSame(0, queue~queued)
  self~assertSame("Line1", array[1])

  -- and the overwrite protection
  queue~queue('Line1')
  queue~queue('')
  queue~queue('Line3')

  address io 'NOBLANKOUTPUT' with input using(queue) output using(queue)
  self~assertSame(2, queue~queued)
  self~assertSame("Line1", queue~pull)
  self~assertSame("Line3", queue~pull)

  queue~delete


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
