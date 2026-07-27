/* extracted from SysUnicode::test_fromunicode_codepages */
::routine main public
  -- symbolic code page names, don't test SYMBOL or UTF7
  do cp over "ACP CONSOLE MACCP OEMCP THREAD_ACP"~subWords
    self~assertSame("0 A", SysFromUnicode(.Unicode~A, cp, , , s.) s.!text, "code page" cp)
  end
  -- CP_UTF8 is missing prior to Windows 10 2018
  self~assertSame("0 A", SysFromUnicode(.Unicode~A, "UTF8", , , s.) s.!text, -
   "UTF8 is known to fail prior to Windows 10 2018")
  -- code page numbers (just a small selection of many possible)
  do cp over 437, 850, 1252, 65001
    self~assertSame("0 A", SysFromUnicode(.Unicode~A, cp, , , s.) s.!text)
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
