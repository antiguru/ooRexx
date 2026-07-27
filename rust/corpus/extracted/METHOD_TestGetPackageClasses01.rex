/* extracted from METHOD::TestGetPackageClasses01 */
::routine main public
  tester = .METHODTester~new
  routine = .Routine~new("Testing", "return 123")
  package = routine~package
  classes = tester~TestGetPackageClasses(package)
  self~assertSame(0, classes~items)
  source = .array~new
  source~append("::class a1")
  source~append("::class a2")
  routine = .Routine~new("Testing", source)
  package = routine~package
  classes = tester~TestGetPackageClasses(package)
  self~assertSame(2, classes~items)
  self~assertTrue(classes~hasIndex("A1"))
  self~assertTrue(classes["A1"]~isA(.class))
  self~assertTrue(classes~hasIndex("A2"))
  self~assertTrue(classes["A2"]~isA(.class))

  source = .array~new
  source~append("::class a1 public")
  source~append("::class a2")
  routine = .Routine~new("Testing", source)
  package = routine~package
  classes = tester~TestGetPackageClasses(package)
  self~assertSame(2, classes~items)
  self~assertTrue(classes~hasIndex("A1"))
  self~assertTrue(classes["A1"]~isA(.class))
  self~assertTrue(classes~hasIndex("A2"))
  self~assertTrue(classes["A2"]~isA(.class))

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
