/* extracted from File::test_temporaryPath_environment_variables_windows */
::routine main public
  if \.RexxInfo~platform~caselessStartsWith("Windows") then
    return

  tmp = value("TMP", "c:\testtmp", "environment")
  self~assertSame("c:\testtmp", .File~temporaryPath~string)
  call value "TMP", .nil, "environment"

  -- if TMP isn't set, TEMP is checked
  temp = value("TEMP", "c:\testtemp", "environment")
  self~assertSame("c:\testtemp", .File~temporaryPath~string)
  call value "TEMP", .nil, "environment"

  -- if neither TMP nor TEMP is set, USERPROFILE is checked
  user = value("USERPROFILE", "c:\testuser", "environment")
  self~assertSame("c:\testuser", .File~temporaryPath~string)
  call value "USERPROFILE", .nil, "environment"

  -- if neither of those three is set, the current directory is returned
  self~assertSame(directory(), .File~temporaryPath~string)

  -- restore environment
  call value "TMP", (tmp == "")~?(.nil, tmp), "environment"
  call value "TEMP", (temp == "")~?(.nil, temp), "environment"
  call value "USERPROFILE", (user == "")~?(.nil, user), "environment"

-- on Unix environment variable TMPDIR is checked
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
