/* extracted from SysUnicode::test_fromunicode_flags_no_compositecheck */
::routine main public
  -- The three flags DEFAULTCHAR, DISCARDNS, and SEPCHARS all
  -- require that COMPOSITECHECK be specified together with each.
  -- Otherwise the call fails with 1004 "Invalid flags"
  -- (results vary with different codepages, but still ...)
  self~assertSame(1004, SysFromUnicode(.Unicode~A, 437, "DEFAULTCHAR", , s.))
  self~assertSame(1004, SysFromUnicode(.Unicode~A, 437, "DISCARDNS", , s.))
  self~assertSame(1004, SysFromUnicode(.Unicode~A, 437, "SEPCHARS", , s.))

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
