/* extracted from DO::test_DO_standardTest2G */
::routine main public
   wn=12; i=0
   DO wn;  i=i+1 ;END                        /* exprr= n            */
   self~AssertSame(i, 12)
   i=0
   DO j=0 by wn to 144; i=i+1 ;END           /* exprb= wn           */
   self~AssertSame(i, 13)
   i=0
   DO j=0 to wn ; i=i+1 ;END                 /* exprt=wn            */
   self~AssertSame(i, 13)
   i=0
   DO j=0 to 12 until i>wn; i=i+1 ;END      /* expru=wn            */
   self~AssertSame(i, 13)
   i=0
   DO j=0 to 12 while i<=wn; i=i+1 ;END      /* exprw=wn            */
   self~AssertSame(i, 13)
   NUMERIC DIGITS 1
   i=0; k=-9
   DO j=0 by wn to 144; i=i+1; k=k+1; If k=8 Then Leave; End;
   self~AssertSame((j i k wn), (1E+2 1E+1 8 12))
   i=0
   DO j=0 to wn ; i=i+1 ; If i= 1E+1 Then Leave; End;
   self~AssertSame((j i wn), (5 6 12))
   i=0
   DO j=0 to 12 until i>wn ; i=i+1 ;If i= 1E+1 Then Leave; End;
   self~AssertSame((j i wn), (5 6 12))
   i=0
   DO j=0 to 12 while i<=wn; i=i+1 ; If i=1E+1 Then Leave; End;
   self~AssertSame((j i wn), (5 6 12))

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
