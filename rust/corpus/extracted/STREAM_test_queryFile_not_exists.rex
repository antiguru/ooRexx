/* extracted from STREAM::test_queryFile_not_exists */
::routine main public

  fileName = "zzaa___tt"
  -- Make sure this doesn't exist for some bizarre reason.
  self~assertFalse(SysFileExists(fileName))

  ret = stream(fileName, 'c', 'query datetime')
  self~assertSame("", ret)

  ret = stream(fileName, 'c', 'query exists')
  self~assertSame("", ret)

  ret = stream(fileName, 'c', 'query handle')
  self~assertSame("", ret)

  ret = stream(fileName, 'c', 'query seek')
  self~assertSame("", ret)

  ret = stream(fileName, 'c', 'query position')
  self~assertSame("", ret)

  ret = stream(fileName, 'c', 'query size')
  self~assertSame("", ret)

  ret = stream(fileName, 'c', 'query streamtype')
  self~assertSame("UNKNOWN", ret)

  ret = stream(fileName, 'c', 'query timestamp')
  self~assertSame("", ret)


-- Tests the command 'query' option, with an existing directory.
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
