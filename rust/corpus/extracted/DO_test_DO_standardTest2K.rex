/* extracted from DO::test_DO_standardTest2K */
::routine main public
   SELECT;
      WHEN i=k THEN self~assertFalse(1)
      WHEN i=j Then self~assertFalse(1)
      OTHERWISE     self~assertTrue(1)
      END;
   drop a; i=1;k=1;                    /* force NOVALUE             N0012*/
   SELECT;
      WHEN i=k THEN Do;            /* NOVALUE in THEN                */
         self~assertTrue(1);End;
      OTHERWISE self~assertFalse(1)
   END;
   drop a; i=1;k=2;                    /* force NOVALUE             N0013*/
   SELECT;
      WHEN i=k THEN self~assertFalse(1)
      OTHERWISE     Do;            /* Novalue in OTHERWISE           */
         self~assertTrue(1);End;
      END;

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
