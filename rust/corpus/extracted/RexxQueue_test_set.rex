/* extracted from RexxQueue::test_set */
::routine main public
  q = .RexxQueue~new
  old = q~get
  -- will happily set to any (including invalid) name
  do name over "session", "", "*", "test_set"
    self~assertSame(old, q~set(name)) -- returns old name
    self~assertSame(name~upper, q~get) -- new name
    old = name~upper
  end
  -- set doesn't create external queue
  self~assertFalse(.RexxQueue~exists(name))

  -- change an instance to another external queue
  q1 = .RexxQueue~new(name)
  q2 = .RexxQueue~new("")
  q1~lineOut("line")
  q2~set(name)
  self~assertEquals("line", q2~pull)
  self~assertSame(0, q1~delete)


-- queued

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
