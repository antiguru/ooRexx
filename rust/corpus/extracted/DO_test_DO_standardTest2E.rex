/* extracted from DO::test_DO_standardTest2E */
::routine main public
   i=0;
   Do 7       ; i=i+1; End;                  /* exprr = constant         */
   self~AssertSame(i, 7)
   i=0; exprr=7;
   Do exprr   ; i=i+1; End;                  /* exprr = variable         */
   self~AssertSame(i, 7)
   i.=0; exprr.4=4;exprr.5=5;exprr.6=6;
   Do j=4 to 6
      Do exprr.j ; i.j=(i.j)+1; End;         /* exprr = compound variable*/
   End
   self~AssertSame((i.4 i.5 i.6), (4 5 6))
   i=0;
   Do ((7*exprr)+1)/exprr.5 ; i=i+1; End;    /* exprr = expression       */
   self~AssertSame(i, 10)

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
