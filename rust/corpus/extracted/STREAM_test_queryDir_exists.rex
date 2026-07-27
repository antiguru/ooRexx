/* extracted from STREAM::test_queryDir_exists */
::routine main public

  dirName = 'TempTestingDir'

  -- Make sure this doesn't exist
  call SysRmDir dirName
  self~assertFalse(SysFileExists(dirName), "directory" dirName "should not exist")

  now = .DateTime~new
  -- wait a bit if SysMkDir might run in the next second
  do while now~microseconds > 950000
    call SysSleep 0.05
    now = .DateTime~new
  end
  self~assertSame(0, SysMkDir(dirName))
  parse value now~isoDate with d 'T' t '.' .
  eTimeStamp = d t
  eDateTime = d~substr(6) || '-' || d~substr(3, 2) t

  ret = stream(dirName, 'c', 'query datetime')
  self~assertSame(eDateTime, ret)

  ret = stream(dirName, 'c', 'query exists')
  self~assertSame("", ret)

  ret = stream(dirName, 'c', 'query handle')
  self~assertSame("", ret)

  ret = stream(dirName, 'c', 'query seek')
  self~assertSame("", ret)

  ret = stream(dirName, 'c', 'query position')
  self~assertSame("", ret)

  ret = stream(dirName, 'c', 'query size')
  self~assertSame(0, ret)

  ret = stream(dirName, 'c', 'query streamtype')
  self~assertSame("UNKNOWN", ret)

  ret = stream(dirName, 'c', 'query timestamp')
  self~assertSame(eTimeStamp, ret)

  j = SysRmDir(dirName)


-- Tests the command 'query' option, with a non-existent directory.
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
