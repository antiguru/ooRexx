/* extracted from Object::test_setmethod_scope */
::routine main public
  t1 = .setTester~new
  t1~testValue = 'abc'
  t1~testSet('gettestvalue', ("expose testValue", "return testValue"))
  self~assertEquals('TESTVALUE', t1~gettestvalue)
  t1~testSet('settestvalue', ("expose testValue", "testValue = 123"))
  t1~setTestValue
  self~assertEquals(123, t1~gettestvalue)
  self~assertEquals('abc', t1~testvalue)

  -- this explicitly sets the default...the results should be the same as above
  t1 = .setTester~new
  t1~testValue = 'abc'
  t1~testSet('gettestvalue', ("expose testValue", "return testValue"), 'FLOAT')
  self~assertEquals('TESTVALUE', t1~gettestvalue)
  t1~testSet('settestvalue', ("expose testValue", "testValue = 123"), 'float')
  t1~setTestValue
  self~assertEquals(123, t1~gettestvalue)
  self~assertEquals('abc', t1~testvalue)

  -- Now create the method at the object scope (which really should have been named
  -- "CLASS".  This will give it access to the scope variable set by the testValue attribute
  t1 = .setTester~new
  t1~testValue = 'abc'
  t1~testSet('gettestvalue', ("expose testValue", "return testValue"), 'OBJECT')
  self~assertEquals('abc', t1~gettestvalue)
  t1~testSet('settestvalue', ("expose testValue", "testValue = 123"), 'object')
  t1~setTestValue
  self~assertEquals(123, t1~gettestvalue)
  self~assertEquals(123, t1~testvalue)

  -- check that SUPER is getting set correctly for these options
  t1 = .setTester~new
  t1~testSet('getsuper', "return super")
  self~assertSame(.setTester, t1~getsuper)
  t1~testSet('getsuper', "return super", 'Float')
  self~assertSame(.setTester, t1~getsuper)
  t1~testSet('getsuper', "return super", 'Object')
  self~assertSame(.object, t1~getsuper)

  -- now check the scope for the created methods
  t1 = .setTester~new
  t1~testSet('getsuper', "return .context~executable~scope")
  self~assertSame(.nil, t1~getsuper)
  t1~testSet('getsuper', "return .context~executable~scope", 'Float')
  self~assertSame(.nil, t1~getsuper)
  t1~testSet('getsuper', "return .context~executable~scope", 'Object')
  self~assertSame(.setTester, t1~getsuper)

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
