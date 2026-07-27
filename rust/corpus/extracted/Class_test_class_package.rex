/* extracted from Class::test_class_package */
::routine main public
  -- pre-defined classes have package name "REXX"
  self~assertEquals("REXX", .Class~package~name)

  -- subclasses created via subclass() method have package .nil
  self~assertEquals(.nil, .Class~subclass("tst_subcls")~package)

  -- dynamically compiled routine, static class
  self~assertEquals("tst_rtn_cls", -
   .Routine~new("tst_rtn_cls", (("return .tst_cls~package~name", "::class tst_cls")))~call)

  -- dynamically compiled routine, subclass
  self~assertEquals(.nil, -
   .Routine~new("tst_rtn_subcls", (("return .Class~subclass('tst_subcls')~package")))~call)

  -- the current package should have the source file name as its package name
  parse source . . sourceFile
  self~assertEquals(sourceFile, .Context~package~name)

  -- all classes within the current package should have the current package as their package
  do class over .Context~package~classes~allItems
    self~assertEquals(.Context~package, class~package)
  end

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
