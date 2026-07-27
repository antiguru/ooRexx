/* extracted from SysSearchPath::test_searchpath_empty */
::routine main public
  self~assertSame("", SysSearchPath("", ""))
  self~assertSame("", SysSearchPath("", "no-file"))
  self~assertSame("", SysSearchPath("no-variable", ""))
  self~assertSame("", SysSearchPath("no-variable", "no-file"))
  self~assertSame("", SysSearchPath("PATH", ""))
  self~assertSame("", SysSearchPath("PATH", "no-file"))

  self~assertSame("", SysSearchPath("", "", "n"))
  self~assertSame("", SysSearchPath("", "", "C"))
  self~assertSame("", SysSearchPath("", "", "No"))
  self~assertSame("", SysSearchPath("", "", "cur"))

  -- SysSearchPath searches for files only, no directories
  self~assertSame("", SysSearchPath("", "/", "nocurrent"))
  self~assertSame("", SysSearchPath("", "/", "CURRENT"))
  self~assertSame("", SysSearchPath("", "/"))
  self~assertSame("", SysSearchPath("", ".", "C"))
  self~assertSame("", SysSearchPath("", "./."))
  self~assertSame("", SysSearchPath("", directory()))

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
