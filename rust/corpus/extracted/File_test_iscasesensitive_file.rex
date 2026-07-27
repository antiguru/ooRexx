/* extracted from File::test_iscasesensitive_file */
::routine main public
  -- test files in home directory, either $HOME, or %USERPROFILE%
  homeDirectory = value((.RexxInfo~platform == "WindowsNT")~?("USERPROFILE", "HOME"), , "environment")
  homeFiles = .File~new(homeDirectory)~listFiles
  if homeFiles == .nil then
    return
  do file over homeFiles
    if file~isFile then do
      if ("WindowsNT", "DARWIN")~hasItem(.RexxInfo~platform) then
        self~assertFalse(file~isCaseSensitive, file~string "expected to be case-insensitive")
      else
        self~assertTrue(file~isCaseSensitive, file~string "expected to be case-sensitive")
      end
  end

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
