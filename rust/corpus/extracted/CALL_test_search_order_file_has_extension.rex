/* extracted from CALL::test_search_order_file_has_extension */
::routine main public

-- We need a directory with a dot to trigger this bug
my.dir = .TemporaryTestDirectory~new(self,"my.dir")~~create

-- Called program, with an extension
called.rex = .TemporaryTestFile~new(my.dir,"called.rex")~~create("return 'ext'")

s = .ooRexxUnit.directory.separator

-- Call "my.dir\called.rex". This will work.
call (my.dir~absolutePath()s"called.rex")
self~assertSame("ext", result)

-- Now call "my.dir\called". It's extensionless, so that the search order
-- should try a number of extensions. It will fail when the presence
-- of a dot in "my.dir" incorrectly makes ooRexx think that the filename
-- includes an extension.
call (my.dir~absolutePath()s"called")
self~assertSame("ext", result)

called.rex~delete
my.dir~delete
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
