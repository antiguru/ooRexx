/* extracted from Method::testDirectives */
::routine main public
  a = .array~new
  a~append("return testRoutine()")
  a~append("::class test1")
  a~append("::routine testRoutine")
  a~append("  return .test1")
  method = .method~new("Testing", a)
  o = .RunTester~new
  o~setMethod("TESTING", method)
  self~assertTrue(o~testing~isA(.class))
  package = method~package

  self~assertSame(package~classes["TEST1"], o~testing)

  self~assertTrue(method~isGuarded)
  self~assertFalse(method~isProtected)
  self~assertFalse(method~isPrivate)

  method~setunGuarded
  self~assertFalse(method~isGuarded)
  method~setGuarded
  self~assertTrue(method~isGuarded)
  method~setPrivate
  self~assertTrue(method~isPrivate)
  method~setProtected
  self~assertTrue(method~isProtected)

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
