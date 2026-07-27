/* extracted from Class::test_class_define */
::routine main public
  t1 = .testDefine1~new
  self~assertFalse(t1~hasMethod('test1'))
  .testDefine1~define('test1', "return 123")
  self~assertFalse(t1~hasMethod('test1'))
  t2 = .testDefine1~new
  self~assertTrue(t2~hasMethod('test1'))
  self~assertEquals(123, t2~test1)
  t3 = .testDefine2~new
  self~assertTrue(t3~hasMethod('test1'))
  self~assertEquals(123, t3~test1)
  .testDefine2~define('test1', .array~of("return 456"))
  self~assertEquals(123, t3~test1)
  t4 = .testDefine2~new
  self~assertEquals(456, t4~test1)
  .testDefine2~delete('test1')
  self~assertEquals(456, t4~test1)
  t5 = .testDefine2~new
  self~assertEquals(123, t5~test1)
  -- replace the method in the original
  .testDefine1~define('test1', .method~new('test1', "return .context~executable~scope"))
  t1 = .testDefine1~new
  t2 = .testDefine2~new
  self~assertSame(.testDefine1, t1~test1)
  self~assertSame(.testDefine1, t2~test1)
  -- hide test1 in testDefine2
  .testDefine2~define('TEST1')
  t2 = .testDefine2~new
  self~expectSyntax(97.1)
  t2~test1

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
