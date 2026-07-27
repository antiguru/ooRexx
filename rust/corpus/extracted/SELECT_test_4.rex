/* extracted from SELECT::test_4 */
::routine main public
   Do i=3 to 1 by -1
      SELECT   /* comm1 */ ; WHEN i=1 THEN R1='1'    /*     comm2           */
         /* comm3 */   WHEN i=2 Then R2='2'    /*     comm4           */
         /* comm5 */   OTHERWISE R3='3'        /*     comm6           */
         END                                            /*     comm7           */
      SELECT ; /* comm1 */   WHEN i=1 THEN R4='4'    /*     comm2           */
         /* comm3 */   WHEN i=2 Then R5='5'    /*     comm4           */
         /* comm5 */   OTHERWISE R6='6'        /*     comm6           */
         END                                            /*     comm7           */
      End
   self~assertSame((r1 r2 r3 r4 r5 r6), (1 2 3 4 5 6))

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
