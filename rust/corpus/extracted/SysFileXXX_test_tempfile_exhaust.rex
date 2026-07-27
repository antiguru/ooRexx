/* extracted from SysFileXXX::test_tempfile_exhaust */
::routine main public
  files = .Array~new(100)
  template = .File~new("test?tempfile?exhaust", .File~temporaryPath)
  -- clean up before trying to exhaust our template
  call SysFileTree template, files, "o"
  do file over files
    call SysRmDir file
    call SysFileDelete file
  end
  files~empty
  dir = .true -- create either directory or file
  loop
    unique = SysTempFileName(template)
    if unique = "" then
      leave
    file = .File~new(unique)
    files~append(file)
    -- create a file/directory until all possibilities are exhausted
    if dir then
      .Stream~new(file)~~open("write replace")~close
    else
      file~makeDir
    dir = \dir
  end
  -- for a template with two "?" we expect exhaustion after 10 x 10
  self~assertSame(100, files~items)
  -- clean up again
  do file over files
    file~delete
  end

-- (Windows-only) UNC filename tests
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
