/* extracted from CONSTANT::test_mixinclass_definition */
::routine main public

  self~assertEquals("Fred", .test3~s1, "s1_class")
  self~assertEquals("Rick", .test3~s2, "s2_class")
  self~assertEquals(" ", .test3~s3, "s3_class")
  self~assertEquals(" ", .test3~s4, "s4_class")
  self~assertEquals("1", .test3~n1, "n1_class")
  self~assertSame("1.00000000000000000000", .test3~n2, "n2_class")
  self~assertEquals(".23", .test3~n3, "n3_class")
  self~assertEquals("1E4", .test3~n4, "n4_class")
  self~assertEquals("1E-4", .test3~n5, "n5_class")
  self~assertEquals("1E+4", .test3~n6, "n6_class")
  self~assertEquals("23.38885e-5", .test3~n7, "n7_class")
  self~assertEquals(".NIL", .test3~c1, "c1_class")
  self~assertEquals(".", .test3~c2, "c2_class")

  t = .test3~new

  self~assertEquals("Fred", t~s1, "s1_instance")
  self~assertEquals("Rick", t~s2, "s2_instance")
  self~assertEquals(" ", t~s3, "s3_instance")
  self~assertEquals(" ", t~s4, "s4_instance")
  self~assertEquals("1", t~n1, "n1_instance")
  self~assertSame("1.00000000000000000000", t~n2, "n2_instance")
  self~assertEquals(".23", t~n3, "n3_instance")
  self~assertEquals("1E4", t~n4, "n4_instance")
  self~assertEquals("1E-4", t~n5, "n5_instance")
  self~assertEquals("1E+4", t~n6, "n6_instance")
  self~assertEquals("23.38885e-5", t~n7, "n7_instance")
  self~assertEquals(".NIL", t~c1, "c1_instance")
  self~assertEquals(".", t~c2, "c2_instance")

  self~assertEquals("Fred", t~class~s1, "s1_instance_class")
  self~assertEquals("Rick", t~class~s2, "s2_instance_class")
  self~assertEquals(" ", t~class~s3, "s3_instance_class")
  self~assertEquals(" ", t~class~s4, "s4_instance_class")
  self~assertEquals("1", t~class~n1, "n1_instance_class")
  self~assertSame("1.00000000000000000000", t~class~n2, "n2_instance_class")
  self~assertEquals(".23", t~class~n3, "n3_instance_class")
  self~assertEquals("1E4", t~class~n4, "n4_instance_class")
  self~assertEquals("1E-4", t~class~n5, "n5_instance_class")
  self~assertEquals("1E+4", t~class~n6, "n6_instance_class")
  self~assertEquals("23.38885e-5", t~class~n7, "n7_instance_class")
  self~assertEquals(".NIL", t~class~c1, "c1_instance_class")
  self~assertEquals(".", t~class~c2, "c2_instance_class")
  self~assertEquals("3.14159265", t~pi)
  self~assertEquals("3.14159265", t~class~pi)
  self~assertEquals("3.14159265"/2, t~piOverTwo)
  self~assertEquals("3.14159265"/2, t~class~piOverTwo)

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
