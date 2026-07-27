/* extracted from METHOD::testAttributeMethods */
::routine main public
  t = .methodTest~new
  self~assertTrue(t~hasmethod('anattribute'))
  self~assertTrue(t~hasmethod('anattribute='))
  getter = t~instanceMethod('ANATTRIBUTE')
  setter = t~instanceMethod('ANATTRIBUTE=')
  self~assertTrue(getter~isAttribute)
  self~assertTrue(setter~isAttribute)

  self~assertEquals("ANATTRIBUTE", t~anAttribute)
  t~anattribute = .nil
  self~assertTrue(\var('RESULT'))
  self~assertSame(.nil, t~anattribute)
  t~setAnAttribute(.array)
  self~assertSame(.array, t~anattribute)
  t~anattribute = .object
  self~assertSame(.object, t~getanattribute)

  self~assertTrue(t~hasmethod('privateAttribute'))
  self~assertTrue(t~hasmethod('privateAttribute='))

  t~setprivateAttribute(.class)
  self~assertSame(.class, t~getPrivateAttribute)

  self~assertTrue(t~hasmethod('packageattribute'))
  self~assertTrue(t~hasmethod('packageattribute='))
  getter = t~instanceMethod('PACKAGEATTRIBUTE')
  setter = t~instanceMethod('PACKAGEATTRIBUTE=')
  self~assertTrue(getter~isAttribute)
  self~assertTrue(setter~isAttribute)
  self~assertTrue(getter~isPackage)
  self~assertTrue(setter~isPackage)

  self~assertEquals("PACKAGEATTRIBUTE", t~packageAttribute)
  t~packageAttribute = .nil
  self~assertTrue(\var('RESULT'))
  self~assertSame(.nil, t~packageAttribute)

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
