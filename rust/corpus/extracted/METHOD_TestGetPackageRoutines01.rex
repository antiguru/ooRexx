/* extracted from METHOD::TestGetPackageRoutines01 */
::routine main public
  tester = .METHODTester~new
  routine = .Routine~new("Testing", "return 123")
  package = routine~package
  routines = tester~TestGetPackageRoutines(package)
  self~assertSame(0, routines~items)
  source = .array~new
  source~append("::routine a1")
  source~append("  return 1")
  source~append("::routine a2")
  source~append("  return 2")
  routine = .Routine~new("Testing", source)
  package = routine~package
  routines = tester~TestGetPackageRoutines(package)
  self~assertSame(2, routines~items)
  self~assertTrue(routines~hasIndex("A1"))
  self~assertTrue(routines["A1"]~isA(.routine))
  self~assertSame(1, routines["A1"]~call)
  self~assertTrue(routines~hasIndex("A2"))
  self~assertTrue(routines["A2"]~isA(.routine))
  self~assertSame(2, routines["A2"]~call)

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
