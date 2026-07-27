/* extracted from SysWin_xxx_Printer::test_SetDefaultPrinter_ret_old_style */
::routine main public

  -- First be reasonably sure we have the default printer.  Then test.
  printer = SysWinGetDefaultPrinter()

  -- The format for the returned printer, should have 2 commas in it.  This could
  -- always change in newer versions of Windows.
  self~assertTrue(printer~isA(.string))
  self~assertSame(2, printer~countStr(","))

  -- Now set the default printer to be what it already is.  That way this test
  -- should not mess up anyones system.
  --
  -- Note that sometimes the return code is 1460, which is an operating system
  -- error code:
  --  ERROR_TIMEOUT This operation returned because the timeout period expired
  --  This is perfectly acceptable and shows the routine is handling errors ok.

  ret = SysWinSetDefaultPrinter(printer)
  isGoodReturnCode = (ret == 0 | ret == 1460)
  self~assertTrue(isGoodReturnCode, "SysWinSetDefaultPrinter should succeed with a return of 0 or fail with 1460")

  -- Now getting the default printer should match the old default
  printerNew = SysWinGetDefaultPrinter()
  self~assertSame(printer, printerNew, "Old default printer should match new default printer")


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
