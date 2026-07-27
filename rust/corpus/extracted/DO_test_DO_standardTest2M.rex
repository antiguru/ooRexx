/* extracted from DO::test_DO_standardTest2M */
::routine main public
   -- Create our external functions.
   path = .ooRexxUnit.dir || .ooRexxUnit.directory.separator
   src = .array~new()
   src[1] = "Parse Version version"
   src[2] = "Parse Source fn"
   src[3] = "extloop:"
   src[4] = "c=0"
   src[5] = "Do i=1 to 3"
   src[6] = "c=c+1"
   src[7] = "If c=2 Then Return c"
   src[8] = "End"
   src[9] = "Return 99"
   call createFile src, path'SCIDOS2M'

   -- now do the tests
   c=0
   Call intloop
   self~AssertSame(c, 2)
   result=17
   Call SCIDOS2M
   self~AssertSame(RESULT, 2)

   -- now remove the external functions
   call deleteFile path'SCIDOS2M'

   intloop:
   Do i=1 to 3
      c=c+1
      if c=2 Then Return                /* results c=2                    */
      End

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
