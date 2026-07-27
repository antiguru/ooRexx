/* extracted from Object::test_instancemethod */
::routine main public
  -- some tests of instancemethod, instancemethods, and method scope
  t = .test2~new

  s = t~instanceMethods(.test2)
  loop while s~available
     name = s~index
     method = s~item
     self~assertSame(.test2, method~scope)
     self~assertSame(method, t~instanceMethod(name))
     self~assertSame(method, .test2~method(name))
     s~next
  end

  -- because there is an override, we need to get an instance of the
  -- superclass for comparing methods
  t2 = .test1~new

  -- go one level down the hierarchy
  s = t~instanceMethods(.test1)
  loop while s~available
     name = s~index
     method = s~item
     self~assertSame(.test1, method~scope)
     self~assertSame(method, t2~instanceMethod(name))
     self~assertSame(method, .test1~method(name))
     s~next
  end

  -- now get all of the methods and compare them with what the
  -- class reports.

  instanceMethods = .relation~new
  instanceSupplier = t~instanceMethods

  loop while instanceSupplier~available
     instanceMethods~put(instanceSupplier~index, instanceSupplier~item)
     instanceSupplier~next
  end

  classMethods = .relation~new
  classSupplier = .test2~methods

  loop while classSupplier~available
     classMethods~put(classSupplier~index, classSupplier~item)
     classSupplier~next
  end

  self~assertTrue(instanceMethods~equivalent(classMethods))

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
