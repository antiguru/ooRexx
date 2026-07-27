/* extracted from CALL::test_1 */
::routine main public
   -- Create our external functions. There are 3 of them.
   path = .ooRexxUnit.dir || .ooRexxUnit.directory.separator
   src = .array~new()
   src[1] = "Parse Version version"
   src[2] = "Parse Source fn"
   src[3] = "Return '4'"
   call createFile src, path'SCICAL1A'
   src[3] = "Return 3"
   call createFile src, path'SCICAL1B'
   src[3] = "Return 4"
   call createFile src, path'SCICAL1C'

   -- now do the tests
   call junktest; sigl0=sigl /* base to check sigl       */
   call internal1
   Call internal2; s=s||left(result,1); sigl2=sigl
   cALL internal3; sigl3=sigl
   CALL,
        'SCICAL1A'; s=s||result
   call/**/"XRANGE" '5','5'; s=s||result
   call left '6789',1; s=s||result
   call scical1B; s=s||result;
   call scical1C; s=s||result;
   call 15
   Call 15.3
   Call .
   CAll ,
   abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghij
   self~assertSame(s, '12345634DEFG')
   self~assertSame((sigl1 sigl2 sigl3), (sigl0+1) (sigl0+2) (sigl0+3))

   -- now remove the external functions
   call deleteFile path'SCICAL1A'
   call deleteFile path'SCICAL1B'
   call deleteFile path'SCICAL1C'

   return

   internal1: s='1'; sigl1=sigl; Return
   internal2: Procedure; Return '2'
   internal3: Procedure Expose s; s=s'3'; Return
   LEFT: Return "LEFT"("ARG"(1),1); /* internal routine overrides bif   */
                                    /* unless literal as function-name  */
   15:   s=s||'D'; Return
   15.3: s=s||'E'; Return
   .:    s=s||'F'; Return
   abcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghijabcdefghij:
         s=s||'G'; Return 0
   -- this routine is here just so the caller can set sigl
   junktest: return

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
