/* extracted from rxQueue::test_multi_lines */
::routine main public

  -- In order to work on different platforms, it is easier to just create our
  -- own program so we can control the number of lines of output.
  src = .array~new
  src[1] = "do i = 1 to 5"
  src[2] = "  say 'line' i"
  src[3] = "end"
  src[4] = "return 5"

  prg = createRexxPrgFile(src, "TestRxQueue")

  -- Assert the file was created okay
  self~assertNotSame('', prg)

  'rexx "'prg'" | rxqueue'

  -- Delete the temp file
  j = deleteFile(prg)

  -- Now assert what should be true
  count = queued()
  self~assertSame(5, count)

  do i = 1 to 5
    parse pull line
    expected = 'line' i
    self~assertSame(expected, line)
  end

/** test_stderr_to_rxqueue()
 * This tests that stderr can be redirected and both stderr and stdout end up
 * in the queue.  This works on 3.2.0 in Windows and it seems reasonable to
 * ensure it keeps working in the future.  Note, that it is conceivable that
 * this may not work on all platforms that ooRexx works on.
 */
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
