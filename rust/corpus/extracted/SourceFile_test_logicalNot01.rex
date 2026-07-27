/* extracted from SourceFile::test_logicalNot01 */
::routine main public

    fileSrc = .array~new
    fileSrc[1]  = "ret = 9"
    fileSrc[2]  = "day = 'friday'"
    fileSrc[3]  = "if 'friday' �= day then do"
    fileSrc[4]  = "  say 'it is not friday'"
    fileSrc[5]  = "  ret = 1"
    fileSrc[6]  = "end"
    fileSrc[7]  = "else do"
    fileSrc[8]  = "  say 'it IS friday'"
    fileSrc[9]  = "  ret = 0"
    fileSrc[10] = "end"
    fileSrc[11] = "return ret"

    fileName = createRexxPrgFile(fileSrc, "test_logicalNot01_temp")
    self~assertTrue(fileName <> "", "Source file must be created")
    self~assertTrue(SysIsFile(fileName), "Source file must exist")

    output = .array~new
    prgRC = execRexxPrg(fileName, output)
    j = deleteFile(fileName)

    self~assertTrue(prgRC == 0, "Program rc must be 0")
    self~assertTrue(output[1] == "it IS friday", "Program output must be correct")

  -- Test \== using 0xAA (�==) when true
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
