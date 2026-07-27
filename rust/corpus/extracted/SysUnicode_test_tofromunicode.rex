/* extracted from SysUnicode::test_tofromunicode */
::routine main public
  -- UTF-8 text converted to Windows Unicode and back to UTF-8 should return unchanged
  do line over .resources~wikipedia
    self~assertSame("0 0" line, -
     SysToUnicode(line, "utf8", , s.) SysFromUnicode(s.!text, "utf8", , , s.) s.!text, -
      "UTF8 is known to fail prior to Windows 10 2018")
  end

  -- even for a multi-line conversion
  lines = .resources~wikipedia~makeString
  self~assertSame("0 0" lines, -
   SysToUnicode(lines, "utf8", , s.) SysFromUnicode(s.!text, "utf8", , , s.) s.!text)


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
