/* extracted from Class::test_DEFINE */
::routine main public

  .test_a~define("TESTMETHOD", "return 'test'" )  -- define a method
  o1=.test_a~new                 -- create an instance
  self~assertEquals("test", o1~testmethod)
  self~assertTrue(o1~hasmethod("TESTMETHOD"))

  self~assertNotNull(.test_a~method("testmethod"))

      -- "unaccessible" means that from now on the method cannot resolved, even if it existed in a superclass!
  .test_a~define("testmethod")   -- make it unaccessible for new instances
  self~assertNull(.test_a~method("testmethod"))

  self~assertEquals("test", o1~testmethod)
  self~assertTrue(o1~hasmethod("TESTMETHOD"))

  o2=.test_a~new                 -- create an instance
  self~assertFalse(o2~hasmethod("TESTMETHOD"))
  self~assertTrue(test_method_not_available(o2))
  return

test_method_not_available: procedure
  use arg o
  signal on any
  o~testmethod
  return .false
any:
  return .true


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
