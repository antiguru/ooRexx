/* extracted from Package::TestAddPublicRoutine01 */
::routine main public

  package = .package~new("ADDPUBLICROUTINETEST1", ("::routine routine1", "  return routine2()"))

  routine = .routine~new("TestingAddPublicRoutine", "return 123")

  package~addPublicRoutine("ROUTINE2", routine)

  routines = package~routines
  self~assertSame(2, routines~items)
  publicRoutines = package~publicRoutines
  self~assertSame(1, publicRoutines~items)
  self~assertSame(123, routines["ROUTINE1"]~call)
  self~assertSame(routine, package~findroutine("ROUTINE2"))
  self~assertSame(routine, package~findpublicroutine("ROUTINE2"))

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
