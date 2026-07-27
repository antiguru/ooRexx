/* extracted from File::test_file_timestamps */
::routine main public
  file = .TemporaryTestFile~new(self, "file_timestamps")~~create("")

  y = .Datetime~new~year
  do date over "1969-12-31", "1970-01-01", y || "-01-01", y || "-02-28", y || "-08-08", y || "-12-31"
    -- we test 24 hours of two time timestamps:
    -- 1. YYYY-MM-DD hh:00:00, with hh = 00 .. 23
    d00 = .DateTime~fromStandardDate(date, "-")
    -- 2. YYYY-MM-DD hh:59:59, with hh = 00 .. 23
    d59 = d00~addHours(1)~addSeconds(-1)

    -- on FreeBSD and NetBSD, most probably due to the mktime() (time_t)-1
    -- ambiguity, a UTC file timestamp of 1969-12-31T23:59:59 cannot be set
    -- e. g. in a GMT+1 timezone:
    -- touch -m -t 197001010059.59 file
    -- touch: out of range or illegal time specification: [[CC]YY]MMDDhhmm[.SS]
    -- timestamps plus/minus one second work
    -- touch -m -t 197001010059.58 file
    -- touch -m -t 197001010100.00 file
    -- still, lastModified= returns .true, so we test a different second
    if .RexxInfo~platform~caselessStartsWith("NetBSD") | -
       .RexxInfo~platform~caselessStartsWith("FreeBSD"), -
       date == "1969-12-31" | date == "1970-01-01" then
      d59 = d59~addSeconds(-1) -- test hh:59:58 instead, with hh = 00 .. 23

    do 24
      self~assertTrue(file~"lastAccessed="(d00), d00~string)
      self~assertEquals(d00, file~lastAccessed, d00~string)
      self~assertTrue(file~"lastModified="(d59), d59~string)
      self~assertEquals(d59, file~lastModified, d59~string)
      d00 = d00~addHours(1)
      d59 = d59~addHours(1)
    end
  end


-- listRoots tests

-- too many arguments
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
