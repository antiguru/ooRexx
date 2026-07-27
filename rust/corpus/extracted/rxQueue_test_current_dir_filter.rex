/* extracted from rxQueue::test_current_dir_filter */
::routine main public

  -- Use a select to allow for expansion.  When this test fails on a plaform
  -- because cmd is not set correctly, then whoever investigates can add the
  -- appropriate command for that platform.  Note it is possible that we will
  -- also need to test for the appropriate shell.
  os = .ooRexxUnit.OSName
  select
    when os == 'WINDOWS' then cmd = 'cd'
    -- other platforms are expected to be Unix-like and support "pwd"
    otherwise cmd = 'pwd'
  end

  cmd "| rxqueue"
  count = queued()

  self~assertSame(1, count)

  -- Be sure we don't hang on the parse pull. If the assert fails the test ends
  -- and we won't hit the parse pull line.
  self~assertTrue(count > 0)

  parse pull line
  trueCurrentDir = directory()

  -- on a case-insensitive file system, pwd and directory()
  -- might actually return strings with different upper/lower-case
  if .File~new(line)~isCaseSensitive then
    self~assertSame(trueCurrentDir, line)
  else
    self~assertSame(trueCurrentDir~lower, line~lower)

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
