/* extracted from Routine::testCall */
::routine main public
  routine = .routine~new("Test", "return .context~args")
  a = routine[]
  self~assertEquals(a, .array~new(0))
  a = routine~call
  self~assertEquals(a, .array~new(0))

  a = routine["abc"]
  self~assertEquals(a, .array~of("abc"))
  a = routine~call("abc")
  self~assertEquals(a, .array~of("abc"))

  a = routine[1,2,3,4,5]
  self~assertEquals(a, .array~of(1,2,3,4,5))
  a = routine~call(1,2,3,4,5)
  self~assertEquals(a, .array~of(1,2,3,4,5))

  test = .array~new(3)
  test[1] = "Fred"
  test[3] = "Mike"
  a = routine["Fred",,"Mike"]
  self~assertEquals(a, test)
  a = routine~call("Fred",,"Mike")
  self~assertEquals(a, test)

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
