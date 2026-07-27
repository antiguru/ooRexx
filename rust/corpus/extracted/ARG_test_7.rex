/* extracted from ARG::test_7 */
::routine main public
   a2='a2'; a3='a3'; a4='a4'; a5 ='a5'
   a6='a6'; a7='a7'; a8='a8'; a9='a9'
   a1=9
   call argm a1,,a3,,a5,,a7,,a9
   return
   argm:
   self~assertSame("ARG"(), arg(1))
   DO i=2 to "ARG"()
      IF i//2=0 THEN DO;
         self~assertSame("ARG"(i), '')
         END
      ELSE DO;
         self~assertSame("ARG"(i), 'a'||i)
         END
      END
   DO i=i to 20;
      self~assertSame("ARG"(i), '')
      END
   RETURN 0

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
