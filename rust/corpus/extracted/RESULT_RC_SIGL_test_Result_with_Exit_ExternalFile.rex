/* extracted from RESULT_RC_SIGL::test_Result_with_Exit_ExternalFile */
::routine main public
   expose fileName
   call (fileName)      -- call external program

   if var("RESULT") then tmpResult=result -- save value of "result"
   if var("RC")     then tmpRC    =rc     -- save value of "rc"

   self~assertTrue(var("TMPRESULT"))
   self~assertFalse(var("TMPRC"))
   self~assertSame("234", tmpResult)  -- value of EXIT in external file

      -- routine
   self~assertSame("ExitValue", testExternalWithExit())
   self~assertFalse(var("RESULT")) -- must not be set !

   call testExternalWithExit
   self~assertSame("ExitValue", result)
      -- now that a function/method was invoked, Rexx dropped "RESULT"
   self~assertFalse(var("RESULT"))  -- must not be set !

   call testExternalWithExit
   self~assertTrue(var("RESULT"))   -- must be set !


   -- now address external Rexx program as a command, hence retrieving a "RC"-value
   -- instead of a "RESULT"
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
