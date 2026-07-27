/* extracted from PARSE::Test_672 */
::routine main public
   b=''
   cl='124 125 126 255 256 257 2000 '2**24-1
   Do ci=1 To Words(cl)
      c=Word(cl,ci)
      If c>1000000 Then Iterate
      If c>2000 Then Do;
         cc=c%10 ; cr=c//10
         a="COPIES"('1111111110',cc)
         If cr<>0 Then Do j=1 to cr;
            a=a||1;
            End j
         End
      Else Do; cc=c%2;cr=c//2
         a="COPIES"('10',cc)
         If cr<>0 Then a=a||1
         End
      Do i=1 Until a=''
         Parse Var a a.i '0' a
--       If c>1000 Then Do; If i//1000=0 Then Say i a.i; End
--       Else Do; If i//100=0  Then Say i a.i; End
         End i
      self~assertTrue(1)
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
