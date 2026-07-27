/* extracted from RESULT_RC_SIGL::test_Result_with_Exit_ADDRESSING_ExternalFile */
::routine main public
   expose fileName

   currentEnvironment=address()              -- save current environment
   address (ooRexxUnit.getShellName())       -- set environment to shell

   -- ADDRESS CMD "rexx" fileName
   "rexx" fileName

   if var("RESULT") then tmpResult=result -- save value of "result"
   if var("RC")     then tmpRC    =RC     -- save value of "result"

-- say "tmpresult="pp(tmpresult) var("tmpresult")
-- say "tmprc    ="pp(tmprc)     var("tmprc")

   address (currentEnvironment)              -- restore current environment

--say pp(var("TMPRESULT"))
   self~assertFalse(var("TMPRESULT"))
   self~assertTrue(var("TMPRC"))
   self~assertSame("234", tmpRC)  -- value of EXIT in external file


   -- check whether SIGL points to the correct lines
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
