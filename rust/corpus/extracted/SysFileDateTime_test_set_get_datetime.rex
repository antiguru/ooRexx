/* extracted from SysFileDateTime::test_set_get_datetime */
::routine main public
  testFile = .TemporaryTestFile~new(self, "test_set_datetime")~create
  parse value testFile~lastModified~isoDate with date "T" time "."
  test = testFile~absolutePath

  -- set either date or time
  self~assertSame(0, SysSetFileDateTime(test, "2020-04-01"), "SysSetFileDateTime("test", 2020-04-01)")
  -- time must still be unchanged
  self~assertSame("2020-04-01" time, SysGetFileDateTime(test))
  self~assertSame(0, SysSetFileDateTime(test, , "04:05:59"))
  -- now the date must still be unchanged
  self~assertSame("2020-04-01 04:05:59", SysGetFileDateTime(test), "expected to fail on FAT where write time has a resolution of 2 seconds")

  -- set both date and time, winter time
  self~assertSame(0, SysSetFileDateTime(test, "2020-02-02", "15:30:01"))
  self~assertSame("2020-02-02 15:30:01", SysGetFileDateTime(test))
  self~assertSame("2020-02-02 15:30:01", SysGetFileDateTime(test, "write"))

  -- set both date and time, summer time
  self~assertSame(0, SysSetFileDateTime(test, "2020-08-02", "00:30:59"))
  self~assertSame("2020-08-02 00:30:59", SysGetFileDateTime(test))
  self~assertSame("2020-08-02 00:30:59", SysGetFileDateTime(test, "write"))

  -- SysSetFileDateTime sets the modified (write) date/time only
  -- other file timestamps, created and accessed, should be unchanged
  -- Create time is available on Windows only
  if .RexxInfo~platform~caselessStartsWith("Windows") then
    -- If this test is run again after only a few seconds in between,
    -- the following assert may fail on Windows due to the Windows "File
    -- System Tunneling cache" which keeps the creation date of a file
    -- as-is even if it is deleted and later re-created with the same
    -- name. See https://stackoverflow.com/questions/33227149/after-deleting-file-and-re-creating-file-not-change-creation-date-in-windows/33227233
    self~assertSame(date time, SysGetFileDateTime(test, "c"), "Fails on Windows if the test is repeated with only a few seconds in between due to the file system tunneling cache")
  self~assertSame(date time, SysGetFileDateTime(test, "a"))

  -- set without a date or a time arg, sets the current time
  self~assertSame(0, SysSetFileDateTime(test))
  -- the returned timestamp should be (almost) the current time
  -- as SysGetFileDateTime doesn't return fractions of seconds, we
  -- allow a delta of up to 1.1 seconds
  timestamp = SysGetFileDateTime(test)
  fileTime = .DateTime~fromIsoDate(timestamp~replaceAt("T", 11, 1) || ".000000")
  now = .DateTime~new
  delta = (now - fileTime)~totalSeconds
  self~assertTrue(delta > 0 & delta < 1.1, "file time" timestamp", current time" now~string)

  self~AssertSame(0, SysSetFileDateTime(test, "2020-03-04", "05:06:07"), "SysSetFileDateTime("test", 2020-03-04, 05:06:07)")
  self~AssertSame("2020-03-04 05:06:07", SysGetFileDateTime(test, "write"), "SysGetFileDateTime("test", write)")


  testFile~delete


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
