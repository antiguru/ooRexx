/* extracted from QUALIFY::test_lower_dirs */
::routine main public

  sl = .ooRexxUnit.directory.separator
  parse source . . file

  pathPart = filespec('L', file)

  ret = SysMkDir('testOne')
  self~assertSame(0, ret)
  ret = SysMkDir('testOne'sl'testTwo')
  self~assertSame(0, ret)
  testFileName = 'test.file'
  src = .array~of(" ", " ")

  expected = createFile(src, 'testOne'sl'testTwo'sl || testFileName)

  fileToQualify = 'testOne'sl'testTwo'sl || testFileName

  self~assertSame(expected, qualify(fileToQualify))

  ret = deleteFile(expected)
  ret = SysRmDir('testOne'sl'testTwo')
  ret = SysRmDir('testOne')


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
