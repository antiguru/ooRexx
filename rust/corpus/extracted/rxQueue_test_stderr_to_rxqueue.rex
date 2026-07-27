/* extracted from rxQueue::test_stderr_to_rxqueue */
::routine main public

  -- Note to someone investigating a failure here.  Add the code to get this
  -- to work on your platform.

  -- This redirects stderr correctly on Windows, Linux (sh), and AIX (ksh).
  -- We may need to add a test for the proper shell in use.
  stdErrToStdOut = '2>&1'

  src = .array~new
  src[1] = ".stdout~lineout('stdout line 1')"
  src[2] = ".stdout~lineout('stdout line 2')"
  src[3] = ".stderr~lineout('stderr line 3')"
  src[4] = ".stderr~lineout('stderr line 4')"
  src[5] = "return 0"

  prg = createRexxPrgFile(src, "TestRxQueue")
  self~assertNotSame('', prg)

  'rexx "'prg'"' stdErrToStdOut '| rxqueue'
  j = deleteFile(prg)

  -- Now test.
  count = queued()
  self~assertSame(4, count)

  parse pull line
  self~assertSame("stdout line 1", line)
  parse pull line
  self~assertSame("stdout line 2", line)
  parse pull line
  self~assertSame("stderr line 3", line)
  parse pull line
  self~assertSame("stderr line 4", line)


-- test maximum RXQUEUE line length allowed
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
