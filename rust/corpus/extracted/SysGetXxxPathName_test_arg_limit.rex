/* extracted from SysGetXxxPathName::test_arg_limit */
::routine main public
  -- create a temporary file xx..xx so that currentDir\xx..xx has a length of 259
  longName = "x"~copies(259 - directory()~length - 1)
  longFile = .File~new(longName)
  longStream = .Stream~new(longFile)
  self~assertEquals("READY:", longStream~open("write replace"))
  longStream~close
  self~assertTrue(longFile~exists, "zero-length temp file should have been created")

  path = longFile~absolutePath
  long = SysGetLongPathName(path)
  short = SysGetShortPathName(path)
  self~assertEquals(path, long)
  self~assertEquals(259, long~length)
  self~assertEquals(SysGetShortPathName(directory())~length + 1 + 8, short~length)
  self~assertEquals(long, SysGetLongPathName(SysGetShortPathName(long)))

  -- remove temporary file
  longFile~delete


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
