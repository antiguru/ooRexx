/* extracted from SysUnicode::test_fromunicode_defaultchar */
::routine main public
  -- defaultchar can be set independently of COMPOSITECHECK DEFAULTCHAR flags
  self~assertSame("0 # 1", SysFromUnicode(.Unicode~Psi, 1252, , "#", s.) s.!text s.!useddefaultchar)
  self~assertSame(0 '00'x 1, SysFromUnicode(.Unicode~Psi, 1252, , '00'x, s.) s.!text s.!useddefaultchar)
  -- can be either omitted or a null string
  self~assertSame("0 ? 1", SysFromUnicode(.Unicode~Psi, 1252, , , s.) s.!text s.!useddefaultchar)
  self~assertSame("0 ? 1", SysFromUnicode(.Unicode~Psi, 1252, , "", s.) s.!text s.!useddefaultchar)


-- SysToUnicode tests

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
