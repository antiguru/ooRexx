/* extracted from Package::testImportedClasses01 */
::routine main public
  file = .TemporaryTestFile~new(self, "ImportedClassTest1.cls")
  file~create(("::class class1", "::class class2 public"))

  -- load this package from the file
  package = .package~new(file~fullName)

  package2 = .package~new("importedtesting1", ("return 12", "::requires '"file~fullName"'"))

  classes = package2~importedclasses
  self~assertSame(1, classes~items)
  self~assertTrue(classes['CLASS2']~isA(.class))

  -- add the orginal package to this created one and test the imports
  package2 = .package~new("importedtesting3", "return 12")
  package2~addPackage(package)

  classes = package2~importedclasses
  self~assertSame(1, classes~items)
  self~assertTrue(classes['CLASS2']~isA(.class))

  self~assertSame(package, package2~importedPackages[1])

  -- now a package with no public classes
  file = .TemporaryTestFile~new(self, "ImportedClassTest2.cls")
  file~create(("::class class1", "::class class2"))

  -- load this package from the file
  package = .package~new(file~fullname)

  package2 = .package~new("importedtesting2", ("return 12", "::requires '"file~fullname"'"))
  classes = package2~importedclasses
  self~assertSame(0, classes~items)

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
