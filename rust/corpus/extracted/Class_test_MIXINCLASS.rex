/* extracted from Class::test_MIXINCLASS */
::routine main public
      -- test counter metaclass
   cl=.object~mixinclass("test_mixin_01", .counter)
   i=5
   do i
      cl~new      -- create an object
   end
   self~assertTrue(cl~counter=5)  -- test whether five instances have been created


      -- test counter metaclass, enhance class methods
   cl=.object~mixinclass("test_mixin_02", .counter, .methods)
   str=cl~floating_Method_1    -- is class method available?
   i=5
   do i
      a1=cl~new      -- create an object
   end
   self~assertTrue(cl~counter=5)  -- test whether five instances have been created
   self~assertEquals(str, cl~fm_object)


      -- test singleton metaclass
   cl=.object~mixinclass("test_mixin_03", .singleton)
   i=5
   a1=cl~new
   do i
      a2=cl~new      -- create an object
   end
   self~assertSame(a1, a2)  -- test whether instances are singletons


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
