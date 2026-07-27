/* extracted from DO::test_DO_standardTest2A */
::routine main public
   i=0
   Do cv=13; If i>10 Then Leave; i=i+1; End;
   self~AssertSame((cv i), (24 11))
   i=0;j=1
   Do cv.j= 13; If i>10 Then Leave; i=i+1; End;
   self~AssertSame((cv.j i), (24 11))
   i=0;
   Do cv. = 13; If i>10 Then Leave; i=i+1; End;
   self~AssertSame((cv. i), (24 11))
   b=0                                   /* cv=expression    */
   Do cv=1+3; IF b=1; Then Leave; b=1; End
   self~AssertSame(cv, 5)
   o='';C=0                                          /* are funct.calls  */
   Do cv=f('I',1) By f('B',2) To F('T',3) For F('F',4); c=c+1; End;
   self~AssertSame((cv O C), '5 IBTF 2')
   o='';C=0                                          /*different sequence*/
   Do cv=f('I',1) By f('B',2) For F('F',40) To F('T',3); c=c+1; End;
   self~AssertSame((cv O C), '5 IBFT 2')
   Do i=010     To  1 By 3; Call err;      End
   self~AssertSame(i, '10')
   Do i=010     To 11 While i=0; Call err; End
   self~AssertSame(i, '10')
   Do i=010     To  1 Until 3=3; Call err; End
   self~AssertSame(i, '10')
   Do i=010     To 30 Until i>10;          End
   self~AssertSame(i, '11')
   Do i=010     To 30 Until i<100;         End
   self~AssertSame(i, '10')
   Do i=010e+00 To 20 By -3; Call err;     End
   self~AssertSame(i, '10')
   Do i=1       To 20 For 0; Call err;     End
   self~AssertSame(i, '1')
   Do i=1       To 20 For 1;               End
   self~AssertSame(i, '2')

   f: o=o||"ARG"(1); Return "ARG"(2)

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
