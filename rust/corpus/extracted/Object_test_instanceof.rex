/* extracted from Object::test_instanceof */
::routine main public
  self~assertTrue(.object~isInstanceOf(.object))
  self~assertTrue(.object~isa(.object))

  self~assertTrue(.class~isInstanceOf(.object))
  self~assertTrue(.class~isa(.object))

  self~assertTrue(.class~isInstanceOf(.class))
  self~assertTrue(.class~isa(.class))

  self~assertTrue("abc"~isInstanceOf(.String))
  self~assertTrue("abc"~isA(.String))

  self~assertTrue("abc"~isInstanceOf(.Comparable))
  self~assertTrue("abc"~isA(.Comparable))

  self~assertFalse(.String~isInstanceOf(.String))
  self~assertFalse(.String~isA(.String))

  self~assertTrue(.String~isInstanceOf(.Class))
  self~assertTrue(.String~isA(.Class))

  -- some special tests for integer objects and numberstring objects

  self~assertTrue(.true~isInstanceOf(.String))
  self~assertTrue(.true~isA(.String))

  self~assertTrue((1/3)~isInstanceOf(.String))
  self~assertTrue((1/3)~isA(.String))

  -- some mixin tests
  self~assertTrue(.array~new~isInstanceOf(.collection))
  self~assertTrue(.array~new~isA(.collection))
  self~assertTrue(.array~new~isInstanceOf(.orderedcollection))
  self~assertTrue(.array~new~isA(.orderedcollection))

  self~assertTrue(.directory~new~isInstanceOf(.collection))
  self~assertTrue(.directory~new~isA(.collection))

  self~assertTrue(.directory~new~isInstanceOf(.mapcollection))
  self~assertTrue(.directory~new~isA(.mapcollection))

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
