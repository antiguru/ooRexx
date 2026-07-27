/* extracted from DO::test_DO_standardTest2F */
::routine main public
   i=0;
   Do 7  While i<5 ; i=i+1; End;                       /* exprw=constant */
   self~AssertSame(i, 5)
   i=0; exprw=5
   Do 7  While i<exprw ; i=i+1; exprw=exprw+1; End;    /* exprr=variable */
   self~AssertSame((i exprw), (7 12))
   i.=0; exprw.4=4;exprw.5=5;exprw.6=6;
   Do j=4 to 6
      Do 7  While i.j<exprw.j; i.j=(i.j)+1;End;   /* exprr=compound variable*/
   End
   self~AssertSame((i.4 i.5 i.6), (4 5 6))
   i=0; exprw=5;
   Do 7 While i<((7*exprw)-15)/exprw.5 ; i=i+1; End;  /* exprr=expression*/
   self~AssertSame(i, 4)
   i=0;
   Do 7  Until i> 5 ; i=i+1; End;                      /* expru=constant */
   self~AssertSame(i, 6)
   i=0; expru=5
   Do 7  Until i>=expru ; i=i+1; expru=expru+1;End;    /* expru=variable */
   self~AssertSame((i expru), (7 12))
   expru=5
   i.=0; expru.4=4;expru.5=5;expru.6=6;
   Do j=4 to 6
      Do 7  Until i.j>=expru.j; i.j=(i.j)+1; End;  /*expru=compound variable*/
   End
   self~AssertSame((i.4 i.5 i.6), (4 5 6))
   i=0;
   Do 7 Until i>=((7*expru)-15)/expru.5 ; i=i+1; End; /* expru=expression*/
   self~AssertSame(i, 4)

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
