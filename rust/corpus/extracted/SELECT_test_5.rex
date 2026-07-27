/* extracted from SELECT::test_5 */
::routine main public
   a.=0
   k=0
   v.=0
   v.1.1=1; v.2.2=2; v.3.3=3; v.4.4=4;v.5.5=5
   Do i=1 to 5
      SELECT ;
         WHEN ff(1); THEN Call check(1)
         WHEN ff(2); THEN Call check(2)
         WHEN ff(3); THEN Call check(3)
         WHEN ff(4); THEN Call check(3)     /* force err-then             */
         WHEN ff(8); THEN Call check(5)     /* force err-when             */
         OTHERWISE;       Call check(4)
         END
      /*Call check('after')*/
     END
   Do j=1 To k
      self~assertSame((a.1.j a.2.j a.3.j a.4.j a.5.j), (v.1.j v.2.j v.3.j v.4.j v.5.j))
      End
   return
   check:
   k=k+1
   If i<>ARG(1) Then Do;
      If i<>4 Then self~assertTrue(0)
      End;
   a.i.k=i
   j = i
   Return
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
