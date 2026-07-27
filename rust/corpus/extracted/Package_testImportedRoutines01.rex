/* extracted from Package::testImportedRoutines01 */
::routine main public
  file = .TemporaryTestFile~new(self, "ImportedRoutineTest1.cls")
  file~create(("::routine routine1", "::routine routine2 public"))

  -- load this package from the file
  package = .package~new(file~fullname)

  package2 = .package~new("importedtesting1x", ("return 12", "::requires '"file~fullname"'"))

  routines = package~publicRoutines

  self~assertSame(1, routines~items)
  self~assertTrue(routines['ROUTINE2']~isA(.routine))

  package2 = .package~new("importedtesting3", "return 12")
  package2~addPackage(package)

  routines = package2~importedroutines
  self~assertSame(1, routines~items)
  self~assertTrue(routines['ROUTINE2']~isA(.routine))

  -- now a package with no public routines
  file = .TemporaryTestFile~new(self, "ImportedRoutineTest2.cls")
  file~create(("::routine routine1" , "::routine routine2"))
  package = .package~new(file~fullname)

  package2 = .package~new("importedtesting2x", ("return 12", "::requires '"file~fullname"'"))
  routines = package2~importedroutines
  self~assertSame(0, routines~items)

  self~assertSame(package, package2~importedPackages[1])

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
