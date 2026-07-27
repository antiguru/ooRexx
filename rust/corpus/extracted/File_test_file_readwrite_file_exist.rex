/* extracted from File::test_file_readwrite_file_exist */
::routine main public
  f = .TemporaryTestFile~new(self, "file_exist")
  f~create("line")

  self~assertTrue(f~isFile)
  self~assertFalse(f~isDirectory)
  self~assertTrue(f~canRead)
  self~assertTrue(f~canWrite)

  f~setReadOnly
  self~assertTrue(f~isFile)
  self~assertFalse(f~isDirectory)
  self~assertTrue(f~canRead)
  self~assertFalse(f~canWrite)

  f~setWritable
  self~assertTrue(f~isFile)
  self~assertFalse(f~isDirectory)
  self~assertTrue(f~canRead)
  self~assertTrue(f~canWrite)

-- test with an existing directory
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
