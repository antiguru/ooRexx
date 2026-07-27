/* extracted from File::test_iscasesensitive_fsutil_windows_only */
::routine main public
  if \.RexxInfo~platform~caselessStartsWith("WINDOWS") then
    return

  -- On a recent Windows 10 with enabled Windows Subsystem for Linux, NTFS
  -- allows case-sensitive folders (fsutil file setCaseSensitiveInfo).
  -- fsutil may be unavailable, or may fail if Windows Subsystem for Linux has
  -- not been enabled, or the user doesn't have enough privilege.  For the
  -- privilege issue, the TEMP path is a good place to run the test.
  sensitiveDir = .TemporaryTestDirectory~new(.File~temporaryPath, "test_iscasesensitive_fsutil")~create
  trace off
  signal off error -- don't let ::OPTIONS ALL SYNTAX trigger on non-zero rc
  address "" with output stem ignore. error stem ignore.
  "fsutil file setCaseSensitiveInfo" sensitiveDir~absolutePath "enable"
  if rc \= 0 then do
    sensitiveDir~delete
    return -- can't test, fsutil doesn't work
  end

  -- at this point fsutil should have made this directory case-sensitive
  self~assertTrue(sensitiveDir~isCaseSensitive, sensitiveDir~name "is expected to be case-sensitive")

  -- now create a file and a folder with names differing just in their casing
  -- this should (only) work in a case-sensitive directory
  sensitiveFile = .TemporaryTestFile~new(sensitiveDir, "name")~create("test_iscasesensitive_fsutil")
  sensitiveSubdir = .TemporaryTestDirectory~new(sensitiveDir, "Name")~create

  self~assertTrue(sensitiveFile~isFile, sensitiveFile~name "file should exist")
  self~assertTrue(sensitiveFile~isCaseSensitive, sensitiveFile~name "file should be case-sensitive")

  self~assertTrue(sensitiveSubdir~isDirectory, sensitiveSubdir~name "directory should exist")
  self~assertTrue(sensitiveSubdir~isCaseSensitive, sensitiveSubdir~name "directory should be case-sensitive")

  sensitiveFile~delete
  sensitiveSubdir~delete
  sensitiveDir~delete


-- string, makeString

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
