/* extracted from ANNOTATE::test_package_annotations */
::routine main public
  package = .context~package
  self~assertEquals('Frodo', package~annotation('author'))
  self~assertEquals(2, package~annotation('Version'))
  self~assertEquals("+2", package~annotation('COUNT'))
  self~assertEquals("-1", package~annotation('NeGaTiVe'))
  self~assertEquals(.nil, package~annotation('NotSet'))

  test = .stringtable~of(('AUTHOR', 'Frodo'), ('VERSION', 2), ('COUNT', "+2"), ('NEGATIVE', "-1"))
  self~assertTrue(test~equivalent(package~annotations))

  package~annotations~count = 5
  self~assertEquals("5", package~annotation('COUNT'))

  -- copy the package...this should get a new copy of the annotations table
  newPackage = package~copy
  self~assertEquals(2, newPackage~annotation('Version'))

  -- bump the version.  This should change the copy version but
  -- leave the original alone
  newPackage~annotations~version = 3
  self~assertEquals(3, newPackage~annotation('Version'))
  self~assertEquals(2, package~annotation('Version'))


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
