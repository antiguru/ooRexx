/* extracted from ABBREV::test_20 */
::routine main public
   s="XRANGE"('00'x,'FF'x)
   Do i=0 To 256
      s30="SUBSTR"(s,1,30)
      t="SUBSTR"(s,1,i)
      i1="MAX"(i-1,0)
      i2=i+1
      self~assertSame(.true, ABBREV(s, t, i1))
      self~assertSame(.true, ABBREV(s, t, i))
      self~assertSame(.false, ABBREV(s, t, i2))
      If 15<=i & i<=30 Then
         self~assertSame(.true, ABBREV(s30, t, 15))
      Else
         self~assertSame(.false, ABBREV(s30, t, 15))
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
