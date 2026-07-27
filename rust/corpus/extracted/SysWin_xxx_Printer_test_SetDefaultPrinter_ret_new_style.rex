/* extracted from SysWin_xxx_Printer::test_SetDefaultPrinter_ret_new_style */
::routine main public

  printer = SysWinGetDefaultPrinter()

  self~assertTrue(printer~isA(.string))
  self~assertSame(2, printer~countStr(","))

  parse var printer oldName ',' driver ',' port

  ret = SysWinSetDefaultPrinter(oldName)
  self~assertSame(0, ret, "SysWinSetDefaultPrinter should succeed with a return of 0")

  -- Now getting the default printer should match the old default
  printer = SysWinGetDefaultPrinter()
  parse var printer newName ',' driver ',' port
  self~assertSame(oldName, newName, "Old default printer should match new default printer")

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
