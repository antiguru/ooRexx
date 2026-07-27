/* extracted from SourceFile::test_logicalNot08 */
::routine main public

    fileSrc = .array~new
    fileSrc[1]  = "ret = 9"
    fileSrc[2]  = "day = 'friday'"
    fileSrc[3]  = "if 'friday' ��= day then do"
    fileSrc[4]  = "  say 'it is not friday'"
    fileSrc[5]  = "  ret = 1"
    fileSrc[6]  = "end"
    fileSrc[7]  = "else do"
    fileSrc[8]  = "  say 'it IS friday'"
    fileSrc[9]  = "  ret = 0"
    fileSrc[10] = "end"
    fileSrc[11] = "return ret"

    fileName = createRexxPrgFile(fileSrc, "test_logicalNot08_temp")
    self~assertTrue(fileName <> "", "Source file must be created")
    self~assertTrue(SysIsFile(fileName), "Source file must exist")

    output = .array~new
    prgRC = execRexxPrg(fileName, output)
    j = deleteFile(fileName)

    -- Double �� should produce an error 35.1.  The interpreter will return -35
    -- (the negation of the major code number) and print out the error
    -- diagnositc lines.  The exact error string varies slightly dependent on
    -- platform, the third line should be contain the following string:
    -- Note that if the error messages change, this test will then fail.
    errMsg = "Error 35.1:  Incorrect expression detected"
    errMsgLength = errMsg~length

    -- Note that we assert things are as expected so that if they are not, we
    -- get a test failure, and not an unexpected error.
    self~assertTrue(output~items >= 3, "Expected error output should be at least 3 lines")
    self~assertTrue(output[3]~pos("Error") <> 0, "Expected error line must contain 'Error'")

    -- Note that if the crucial part of the error message changes, this test
    -- will then fail and should be rewritten.
    self~assertSame(errMsg, output[3]~substr(output[3]~pos("Error"), errMsgLength), "Error output must be correct")

    -- Unix/Linux bourne shells will restrict the exit code to the numbers 0 to
    -- 255.   The actual exit code is treated as modulo 256, so -35 will become
    -- 221 on Linux (bash shell.)
    if .ooRexxUnit.OSName == "LINUX" then expectedRC = 221
    else expectedRC = -35

    self~assertSame(expectedRC, prgRC, "Error exit, program rc must be" expectedRC)


-- End of class: SourceFile.testGroup


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
