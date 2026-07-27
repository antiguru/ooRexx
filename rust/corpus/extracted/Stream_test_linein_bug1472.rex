/* extracted from Stream::test_linein_bug1472 */
::routine main public
  bug1472file = "stream_bug_1472.tmp"
  bug1472data = "x" || .String~cr~copies(2)
  -- make data at least 1K in length
  bug1472data = bug1472data~copies(1024 % bug1472data~length + 1)

  s = .Stream~new(bug1472file)
  s~open("write replace")
  s~charOut(bug1472data || .String~cr || .String~nl)
  s~close

  do options over "read shared", "read shared nobuffer"
    s~open(options)
    dataLength = bug1472data~length
    -- step from shorter to longer line lengths to test buffer length increments
    do position = dataLength to 1 by -1
      s~seek(position)
      line = s~lineIn
      self~assertEquals(position, dataLength - line~length + 1)
      self~assertTrue(bug1472data~endsWith(line))
    end
    s~close
  end

  call deleteFile bug1472file



-- End of class: Stream.testGroup


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
