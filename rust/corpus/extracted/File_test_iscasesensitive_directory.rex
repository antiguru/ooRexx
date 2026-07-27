/* extracted from File::test_iscasesensitive_directory */
::routine main public
  -- the class method returns the case sensititvity for "/" or the Windows
  -- system directory
  if .RexxInfo~platform~caselessStartsWith("WINDOWS") then
    root = SysSystemDirectory()
  else
    root = "/"
  self~assertSame(.File~isCaseSensitive, .File~new(root)~isCaseSensitive)

  -- a Windows and a Mac file system typically isn't case-sensitive
  -- (probably not true under all circumstances, but a good guess)
  -- we test root directory, and first directory in PATH
  firstPath = .File~new(value("PATH", , "environment")~makeArray(.File~pathSeparator)[1])
  if ("WindowsNT", "DARWIN")~hasItem(.RexxInfo~platform) then do
    self~assertFalse(.File~isCaseSensitive, "Windows and Mac file system is expected to be case-insensitive")
    self~assertFalse(firstPath~isCaseSensitive, firstPath~string "expected to be case-insensitive")
  end
  else do
    self~assertTrue(.File~isCaseSensitive, "typical Unix file systems is expected to be case-sensitive")
    self~assertTrue(firstPath~isCaseSensitive, firstPath~string "expected to be case-sensitive")
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
