/* extracted from tilde::test_tilde */
::routine main public
  -- create a small file in the user's home directory
  homePath = .File~new("~")~absolutePath
  homeFile = SysTempFilename("~/test_tilde?????")
  self~assertTrue(homeFile~startsWith(homePath))
  homeFileAlternate = homeFile~changeStr(homePath, "~" || userid(), 1)
  homeFile = homeFile~changeStr(homePath, "~", 1)
  .Stream~new(homeFile)~~open("write replace")~~lineOut(homeFile)~close

  -- and another file in the same directory to test
  secondHomeFile = SysTempFilename("~/test_tilde?????")~changeStr(homePath, "~", 1)

  -- make sure our temp files get deleted even if the test case fails
  temporaries = .Array~of(.TemporaryFile~new(homeFile), .TemporaryFile~new(secondHomeFile))

  self~AssertSame(0, SysFileCopy(homeFile, secondHomeFile), "SysFileCopy("homeFile"," secondHomeFile")")
  self~AssertSame(0, SysFileDelete(homeFile), "SysFileDelete("homeFile")")
  self~AssertTrue(   SysFileExists(secondHomeFile), "SysFileExists("secondHomeFile")")
  self~AssertSame(0, SysFileMove(secondHomeFile, homeFile), "SysFileCopy("secondHomeFile"," homeFile")")

  self~AssertTrue(   SysIsFile(homeFile), "SysIsFile("homeFile")")
  self~AssertTrue(   SysIsFile(homeFileAlternate), "SysIsFile("homeFileAlternate")")
  self~AssertTrue(   SysIsFileDirectory("~"), "SysIsFileDirectory(~)")
  self~AssertTrue(   SysIsFileDirectory("~" || userid()), "SysIsFileDirectory(~"userid()")")

  -- create a symlink using SysSymlink() from rxunixsys
  -- unfortunately it doesn't support tilde notation, so we use full paths
  self~AssertSame(0, SysSymlink(homeFile~changeStr("~", homePath), secondHomeFile~changeStr("~", homePath)), "SysSymlink("homeFile"," secondHomeFile")")
  self~AssertTrue(   SysIsFileLink(secondHomeFile), "SysIsFileLink("secondHomeFile")")
  self~AssertSame(0, SysFileDelete(secondHomeFile), "SysFileDelete("secondHomeFile")")

  self~AssertSame(0, SysMkDir(secondHomeFile), "SysMkDir("secondHomeFile")")
  self~AssertSame(0, SysRmDir(secondHomeFile), "SysRmDir("secondHomeFile")")

  homeFileName = filespec("name", homeFile)
  do home over "~", "~" || userid()
    call value "test_tilde", home, "environment"
    self~AssertSame(.File~new(homeFile)~absolutePath, SysSearchPath("test_tilde", homeFileName, "n"), "SysSearchPath(test_tilde," homeFileName", n)")
  end

  self~AssertSame(0, SysSetFileDateTime(homeFile, "2020-03-04", "05:06:07"), "SysSetFileDateTime("homeFile", 2020-03-04, 05:06:07)")
  self~AssertSame("2020-03-04 05:06:07", SysGetFileDateTime(homeFileAlternate, "write"), "SysGetFileDateTime("homeFileAlternate", write)")

  call SysFileDelete homeFile
  call SysFileDelete secondHomeFile


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
