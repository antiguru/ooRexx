/* extracted from whiteSpace::test_TABs */
::routine main public
    TAB  ="09"x
    TAB2 =TAB||TAB

    PLANK=" "
    PLANK2=PLANK||PLANK

      -- test equality of TAB and blank, Rexx-philosophy
    self~assertTrue(PLANK = TAB)
    self~assertTrue(PLANK = "")
    self~assertTrue(""    = TAB)

    self~assertFalse(PLANK ==TAB)
    self~assertFalse(""    ==TAB)
    self~assertFalse(PLANK =="")

    self~assertTrue(PLANK2= TAB2)
    self~assertTrue(""    = TAB2)
    self~assertTrue(PLANK2= "")

    self~assertFalse(PLANK2==TAB2)
    self~assertFalse(""    ==TAB2)
    self~assertFalse(PLANK2=="")

    self~assertEquals(3, words("word1" TAB "word2" "word3"))
    self~assertEquals("a", PLANK "a" PLANK2)
    self~assertEquals("a", PLANK "a" TAB2)
    self~assertEquals("a", TAB "a" TAB2)


   -- starting with 3.2 TAB chars are treated like blanks
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
