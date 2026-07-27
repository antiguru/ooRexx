/* extracted from ANNOTATE::test_attribute_annotation */
::routine main public
  -- lots of different tests here for different ways of creating attributes.
  -- Attributes are complicated because there are two methods involved...but
  -- not necessarily.  We might only have one of the pair, depending on how
  -- the attribute is created.

  -- The first two are for attributes that are defined on the the instance
  -- and class with the same name.  They are annotated separately.  We have
  -- 4 methods to check here
  classGet = .tester~instanceMethod('anAttribute')
  classSet = .tester~instanceMethod('anAttribute=')
  instanceGet = .tester~method('anAttribute')
  instanceSet = .tester~method('anAttribute=')

  self~assertEquals(102, classGet~annotation('VERSION'))
  self~assertEquals(102, classSet~annotation('VERSION'))
  self~assertEquals(101, instanceGet~annotation('VERSION'))
  self~assertEquals(101, instanceSet~annotation('VERSION'))

  -- now the same tests for attributes created using ::method attribute
  classGet = .tester~instanceMethod('aMethodAttribute')
  classSet = .tester~instanceMethod('aMethodAttribute=')
  instanceGet = .tester~method('aMethodAttribute')
  instanceSet = .tester~method('aMethodAttribute=')

  self~assertEquals(1, classGet~annotation('VERSION'))
  self~assertEquals(1, classSet~annotation('VERSION'))
  self~assertEquals(1a, instanceGet~annotation('VERSION'))
  self~assertEquals(1a, instanceSet~annotation('VERSION'))

  -- now attributes where the getter and setter are
  -- specified separately
  instanceGet = .tester~method('anotherAttribute')
  instanceSet = .tester~method('anotherAttribute=')

  self~assertEquals(103, instanceGet~annotation('VERSION'))
  self~assertEquals(103, instanceSet~annotation('VERSION'))

  classGet = .tester~instanceMethod('anotherClassAttribute')
  classSet = .tester~instanceMethod('anotherClassAttribute=')

  self~assertEquals(104, classGet~annotation('VERSION'))
  self~assertEquals(104, classSet~annotation('VERSION'))

  -- now singleton set/get attributes

  -- note, these are different attribute names
  instanceGet = .tester~method('aGetAttribute')
  instanceSet = .tester~method('aSetAttribute=')

  self~assertEquals(106, instanceGet~annotation('VERSION'))
  self~assertEquals(105, instanceSet~annotation('VERSION'))

  classGet = .tester~instanceMethod('aClassGetAttribute')
  classSet = .tester~instanceMethod('aClassSetAttribute=')

  self~assertEquals(108, classGet~annotation('VERSION'))
  self~assertEquals(107, classSet~annotation('VERSION'))

  -- note, these are different attribute names
  instanceGet = .tester~method('split2')
  instanceSet = .tester~method('split1=')

  self~assertEquals(201, instanceGet~annotation('VERSION'))
  self~assertEquals(200, instanceSet~annotation('VERSION'))

  -- the class attributes should be unannotated
  classGet = .tester~instanceMethod('split1')
  classSet = .tester~instanceMethod('split2=')

  self~assertTrue(classGet~annotations~isEmpty())
  self~assertTrue(classGet~annotations~isEmpty())

  -- attributes defined with a code body.  We'll just do the
  -- instance versions here, since all other class vs. instance checks
  -- have passed.  We're just looking to see if this form of attribute
  -- is handled properly.

  instanceGet = .tester~method('codeAttribute')
  instanceSet = .tester~method('codeAttribute=')

  self~assertEquals(202, instanceGet~annotation('VERSION'))
  self~assertEquals(202, instanceSet~annotation('VERSION'))

  -- and finally an attribute defined as abstract...these can still be
  -- annotated

  instanceGet = .tester~method('abstractAttribute')
  instanceSet = .tester~method('abstractAttribute=')

  self~assertEquals(203, instanceGet~annotation('VERSION'))
  self~assertEquals(203, instanceSet~annotation('VERSION'))

-- start of syntax checks

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
