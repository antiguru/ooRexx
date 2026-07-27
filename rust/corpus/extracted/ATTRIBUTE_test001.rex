/* extracted from ATTRIBUTE::test001 */
::routine main public

  -- test how the methods are created
  getmethod = .test~instanceMethod("V1")
  setmethod = .test~instanceMethod("V1=")
  self~assertFalse(getMethod~isPrivate)
  self~assertFalse(getMethod~isPackage)
  self~assertTrue(getMethod~isGuarded)
  self~assertTrue(getMethod~isProtected)
  self~assertTrue(getMethod~isAttribute)
  self~assertFalse(getMethod~isConstant)
  self~assertFalse(getMethod~isAbstract)
  self~assertFalse(setMethod~isPrivate)
  self~assertFalse(setMethod~isPackage)
  self~assertTrue(setMethod~isGuarded)
  self~assertTrue(setMethod~isProtected)
  self~assertTrue(setMethod~isAttribute)
  self~assertFalse(setMethod~isConstant)
  self~assertFalse(setMethod~isAbstract)

  -- test how the methods are created
  getmethod = .test~new~instanceMethod("V1")
  setmethod = .test~new~instanceMethod("V1=")
  self~assertFalse(getMethod~isPrivate)
  self~assertFalse(getMethod~isPackage)
  self~assertFalse(getMethod~isGuarded)
  self~assertFalse(getMethod~isProtected)
  self~assertTrue(getMethod~isAttribute)
  self~assertFalse(getMethod~isConstant)
  self~assertFalse(getMethod~isAbstract)
  self~assertFalse(setMethod~isPrivate)
  self~assertFalse(setMethod~isPackage)
  self~assertFalse(setMethod~isGuarded)
  self~assertFalse(setMethod~isProtected)
  self~assertTrue(setMethod~isAttribute)
  self~assertFalse(setMethod~isConstant)
  self~assertFalse(setMethod~isAbstract)

  getmethod = .test~instanceMethod("V2")
  setmethod = .test~instanceMethod("V2=")
  self~assertFalse(getMethod~isPrivate)
  self~assertFalse(getMethod~isPackage)
  self~assertFalse(getMethod~isGuarded)
  self~assertTrue(getMethod~isProtected)
  self~assertTrue(getMethod~isAttribute)
  self~assertFalse(getMethod~isConstant)
  self~assertFalse(getMethod~isAbstract)
  self~assertFalse(setMethod~isPrivate)
  self~assertFalse(setMethod~isPackage)
  self~assertFalse(setMethod~isGuarded)
  self~assertTrue(setMethod~isProtected)
  self~assertTrue(setMethod~isAttribute)
  self~assertFalse(setMethod~isConstant)
  self~assertFalse(setMethod~isAbstract)

  -- test how the methods are created
  getmethod = .test~new~instanceMethod("V2")
  setmethod = .test~new~instanceMethod("V2=")
  self~assertFalse(getMethod~isPrivate)
  self~assertFalse(getMethod~isPackage)
  self~assertFalse(getMethod~isGuarded)
  self~assertTrue(getMethod~isProtected)
  self~assertTrue(getMethod~isAttribute)
  self~assertFalse(getMethod~isConstant)
  self~assertFalse(getMethod~isAbstract)
  self~assertFalse(setMethod~isPrivate)
  self~assertFalse(setMethod~isPackage)
  self~assertFalse(setMethod~isGuarded)
  self~assertTrue(setMethod~isProtected)
  self~assertTrue(setMethod~isAttribute)
  self~assertFalse(setMethod~isConstant)
  self~assertFalse(setMethod~isAbstract)

  getmethod = .test~instanceMethod("V5")
  setmethod = .test~instanceMethod("V5=")
  self~assertTrue(getMethod~isAttribute)
  self~assertFalse(getMethod~isConstant)
  self~assertTrue(getMethod~isAbstract)
  self~assertTrue(setMethod~isAttribute)
  self~assertFalse(setMethod~isConstant)
  self~assertTrue(setMethod~isAbstract)

  -- test how the methods are created
  getmethod = .test~new~instanceMethod("V5")
  setmethod = .test~new~instanceMethod("V5=")
  self~assertTrue(getMethod~isAttribute)
  self~assertFalse(getMethod~isConstant)
  self~assertTrue(getMethod~isAbstract)
  self~assertTrue(setMethod~isAttribute)
  self~assertFalse(setMethod~isConstant)
  self~assertTrue(setMethod~isAbstract)

  test = .test~new
  self~assertSame("Fred", .test~v1)
  self~assertSame("Rick", test~v1)
  .test~v1 = "Joe"
  self~assertSame("Joe", .test~v1)
  self~assertSame("Rick", test~v1)
  test~v1 = "Larry"
  self~assertSame("Joe", .test~v1)
  self~assertSame("Larry", test~v1)

  self~assertSame("George", .test~v2)
  self~assertSame("David", test~v2)
  .test~v2 = "Curly"
  self~assertSame("Curly", .test~v2)
  self~assertSame("David", test~v2)
  test~v2 = "Moe"
  self~assertSame("Curly", .test~v2)
  self~assertSame("Moe", test~v2)

  self~assertSame("Mike", .test~v3)
  self~assertSame("Mark", test~v3)
  .test~v3 = "Chip"
  self~assertSame("Chip", .test~v3)
  self~assertSame("Mark", test~v3)
  test~v3 = "Walter"
  self~assertSame("Chip", .test~v3)
  self~assertSame("Walter", test~v3)

  self~assertSame("V4", .test~v4)
  self~assertSame("V4", test~v4)
  .test~v4 = "Jon"
  self~assertSame("Jon", .test~v4)
  self~assertSame("V4", test~v4)
  test~v4 = "Brandon"
  self~assertSame("Jon", .test~v4)
  self~assertSame("Brandon", test~v4)

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
