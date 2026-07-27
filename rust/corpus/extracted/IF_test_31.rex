/* extracted from IF::test_31 */
::routine main public
    Do i=0 to 3
       tv.=0
       n=0
       If i>=1 Then Do; tv.1=1;
          n=n+1;
          If i>=2 Then Do; tv.2=1;
             n=n+1
             If i>=3 Then Do; tv.3=1; n=n+1;
                End;
             Else Do; tv.3=2;
                End;
             End;
          Else Do; tv.2=2
             End;
          End
       Else Do; tv.1=2
          End;
       Select
          When i=0 Then self~assertSame((i tv.1 tv.2 tv.3 n), (i 2 0 0 i))
          When i=1 Then self~assertSame((i tv.1 tv.2 tv.3 n), (i 1 2 0 i))
          When i=2 Then self~assertSame((i tv.1 tv.2 tv.3 n), (i 1 1 2 i))
          When i=3 Then self~assertSame((i tv.1 tv.2 tv.3 n), (i 1 1 1 i))
          Otherwise self~assertTrue(0)
          End
      End i

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
