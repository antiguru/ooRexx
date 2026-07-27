/* extracted from RexxInfo::test_rexxInfo_platform */
::routine main public
  self~assertTrue(.RexxInfo~platform~length > 0, ".RexxInfo~platform should return a non-null string")
  parse source platform .
  self~assertEquals(platform, .RexxInfo~platform, ".RexxInfo~platform should be equal to parse source first token")
  -- on Linux, .RexxInfo~platform returns "LINUX", but SysVersion returns "Linux .."
  -- on Windows, .RexxInfo~platform returns "WindowsNT", but SysVersion returns "Windows .."
  parse value SysVersion() with platform .
  self~assertTrue(platform~length > 0, "SysVersion() should return a non-null string")
  if platform~caselessStartsWith("Windows") then
    self~assertTrue(.RexxInfo~platform~caselessStartsWith(platform), ".RexxInfo~platform should match SysVersion() first token")
  else
    self~assertEquals(platform~upper, .RexxInfo~platform~upper, ".RexxInfo~platform should be equal to SysVersion() first token")

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
