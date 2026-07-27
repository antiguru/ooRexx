/* extracted from LINES::test_count_bytes */
::routine main public

      -- This is a repeat of test_normal_bytes, using count, and no comments.
      fileName = createFile(.array~of('dog', 'cat', 'lion', 'tiger'), "delMe_BIF_LinesCountB")
      self~assertFalse(fileName == "", 'temp file must be created')

      bytes = stream(fileName, "C", "QUERY SIZE")
      discard = charin(fileName, bytes)
      self~assertSame(0, lines(fileName, 'C'), 'Read all bytes, use "C" options, 0 lines should remain')

      junk = lineout(fileName)
      junk = deleteFile(fileName)
      fileName = createFile(.array~of('tom', 'frank', 'john', 'harry'), "delMe_BIF_LinesCount2")
      self~assertFalse(fileName == "", 'temp file must be created')
      bytes = stream(fileName, "C", "QUERY SIZE")

      if .ooRexxUnit.OSName~abbrev("WIN") then readBytes = bytes - 2
      else readBytes = bytes - 1

      discard = charin(fileName, readBytes)
      self~assertSame(1, lines(fileName), 'Read all bytes but last newline, "C" option, 1 line should remain')

      -- Delete the file
      junk = lineout(fileName)
      junk = deleteFile(fileName)

    /* The following all test bad syntax options */
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
