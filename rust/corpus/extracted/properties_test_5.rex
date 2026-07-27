/* extracted from properties::test_5 */
::routine main public
   propfile = 'testproperties.txt'
   prop = .properties~new()
   prop['path'] = '/home/me'
   prop['libpath'] = '/home/me/lib'
   prop~save(propfile)
   propfile = 'testproperties.txt'
   prop1 = .properties~new()
   prop1~load(propfile)
   self~assertEquals('/home/me', prop1['path'])
   self~assertEquals('/home/me/lib', prop1['libpath'])
   prop2 = .properties~new()
   prop2['name'] = 'Mike'
   prop2~load(propfile)
   self~assertEquals('/home/me', prop2['path'])
   self~assertEquals('/home/me/lib', prop2['libpath'])
   self~assertEquals('Mike', prop2['name'])
   call SysFileDelete propfile

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
