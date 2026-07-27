/* extracted from ANNOTATE::test_method_annotations */
::routine main public
  -- start with the unattached method
  m = .methods~unattached_method

  self~assertEquals(5, m~annotation('VERSION'))
  self~assertEquals(.nil, m~annotation('NOTTHERE'))

  test = .stringtable~of(('VERSION', 5))
  self~assertTrue(test~equivalent(m~annotations))

  m~annotations~version = 6
  self~assertEquals(6, m~annotation('VERSION'))

  -- copy the method object and verify the annotations
  -- are disconnected.
  newMethod = m~copy

  newMethod~annotations~version = 7
  self~assertEquals(6, m~annotation('VERSION'))
  self~assertEquals(7, newMethod~annotation('VERSION'))

  -- the tester class has both class and instance methods named
  -- unattached_method.  Verify these have picked up the correct
  -- annotations.

  classMethod = .tester~instanceMethod('UNATTACHED_METHOD')
  instanceMethod = .tester~method('UNATTACHED_METHOD')

  -- we've already verified the basics of method annotations...so
  -- just verify that these two methods have picked up the
  -- correct annotation values

  self~assertEquals(4, classMethod~annotation('VERSION'))
  self~assertEquals(3, instanceMethod~annotation('VERSION'))

  -- an method with no annotations
  method = .tester~method('unannotated')
  self~assertEquals(.nil, method~annotation('VERSION'))

  -- not check annotations on abstract methods
  instanceMethod = .tester~method('abstractMethod')
  self~assertEquals(204, instanceMethod~annotation('VERSION'))

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
