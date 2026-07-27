/* extracted from SELECT::test_8 */
::routine main public
   r1=0; r2=0
   i=1 ; j=0                           /* ff() returns 1 for ARG(1)=i    */
   SELECT;
      WHEN ff(1) THEN R1= 1 ;      /*function ref to check If evaluat*/
      OTHERWISE       R2 =2 ;
      END;
   self~assertSame((r1 r2), (1 0))
   return
   ff:                                  /* check,which WHEN is executed?*/
   If ARG(1)<i Then Return 0            /* goto next WHEN       */
   If ARG(1)=i Then Return 1            /* This THEN is wanted  */
   If i=5      Then Return 1            /* forced discrepancy btw.i&ARG */
   self~assertTrue(0)
   return 99

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
