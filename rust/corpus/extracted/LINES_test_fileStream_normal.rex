/* extracted from LINES::test_fileStream_normal */
::routine main public

      -- Create a temp file with exactly 4 lines in it.
      fileName = createFile(.array~of(1, 2, 3, 4), "delMe_BIF_Lines4")

      -- Assert the file was created ok
      self~assertFalse(fileName == "", 'temp file must be created')

      self~assertSame(1, lines(fileName), 'file has lines, omitted 2nd arg should return 1')
      self~assertSame(1, lines(fileName, 'N'), 'file has lines, "N" for 2nd arg should return 1')
      self~assertSame(1, lines(fileName, 'Normal'), 'file has lines, "Normal" for 2nd arg should return 1')

      -- Read 2 lines, should get same results, because file still has lines.
      do 2
        discard = linein(fileName)
      end
      self~assertSame(1, lines(fileName), 'file has 2 lines, omitted 2nd arg should return 1')
      self~assertSame(1, lines(fileName, 'N'), 'file has 2 lines, "N" for 2nd arg should return 1')
      self~assertSame(1, lines(fileName, 'Normal'), 'file has 2 lines, "Normal" for 2nd arg should return 1')

      -- Read 2 lines, should get 0 for same tests.
      do 2
        discard = linein(fileName)
      end
      self~assertSame(0, lines(fileName), 'file has 0 lines, omitted 2nd arg should return 0')
      self~assertSame(0, lines(fileName, 'N'), 'file has 0 lines, "N" for 2nd arg should return 0')
      self~assertSame(0, lines(fileName, 'Normal'), 'file has 0 lines, "Normal" for 2nd arg should return 0')

      -- Be sure file is closed and delete it.
      j = lineout(fileName)
      j = deleteFile(fileName)


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
