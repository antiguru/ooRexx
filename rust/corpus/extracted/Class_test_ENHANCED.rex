/* extracted from Class::test_ENHANCED */
::routine main public

  o=.test_c~enhanced(.methods)      -- create an instance enhanced with the floating methods
  self~assertEquals("set by floating_method_1", o~floating_method_1)
  self~assertEquals("set by floating_method_2", o~floating_method_2)
  self~assertEquals("set by floating_method_3", o~floating_method_3)
  self~assertEquals("set by floating_method_3", o~fm_object)
  tmp="   olah! oho! ah-sooo! "
  o~fm_object=tmp
  self~assertEquals(tmp, o~fm_object)
  self~assertSame(tmp, o~fm_object)

  tmpArg="This is an argument text."
  o2=.test_c~enhanced(.methods, tmpArg)
  self~assertEquals("set by floating_method_1", o2~floating_method_1)
  self~assertEquals("set by floating_method_2", o2~floating_method_2)
  self~assertEquals("set by floating_method_3", o2~floating_method_3)
  self~assertEquals("set by floating_method_3", o2~fm_object)
  tmp="   olah! oho! ah-sooo! "
  o2~fm_object=tmp
  self~assertEquals(tmp, o2~fm_object)
  self~assertSame(tmp, o2~fm_object)

  self~assertEquals(tmpArg, o2~rgf)
  o2~rgf=tmp
  self~assertEquals(tmp, o2~rgf)
  self~assertSame(tmp, o2~rgf)


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
