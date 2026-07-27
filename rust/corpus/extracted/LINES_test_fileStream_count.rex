/* extracted from LINES::test_fileStream_count */
::routine main public

      -- Create a temp file with exactly 64 lines in it.
      txt = .array~new
      do i = 1 to 64
        txt~append(i)
      end
      fileName = createFile(txt, "delMe_BIF_Lines64")
      self~assertFalse(fileName == "", 'temp file must be created')

      self~assertSame(64, lines(fileName, 'C'), 'file has 64 lines, 2nd arg "C" should return 64')
      self~assertSame(64, lines(fileName, 'c'), 'file has 64 lines, 2nd arg "c" should return 64')
      self~assertSame(64, lines(fileName, "count"), 'file has 64 lines, 2nd arg "count" should return 64')

      -- Read 4 lines, option of count should return 60
      do 4
        discard = linein(fileName)
      end
      self~assertSame(60, lines(fileName, 'C'), '60 lines left, 2nd arg "C" should return 60')
      self~assertSame(60, lines(fileName, 'c'), '60 lines left, 2nd arg "c" should return 60')
      self~assertSame(60, lines(fileName, "count"), '60 lines left, 2nd arg "count" should return 60')

      -- Read down to 0 lines, assert count is correct each time
      j = 60
      do i = 1 to 60
        self~assertSame(j, lines(fileName, 'C'), j 'lines left, count option should return' j)
        discard = linein(fileName)
        j -= 1
      end

      self~assertSame(0, lines(fileName, 'COUNT' ), '0 lines left COUNT option should return 0')

      -- Be sure file is closed and delete it.
      junk = lineout(fileName)
      junk = deleteFile(fileName)

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
