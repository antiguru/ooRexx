/* extracted from whiteSpace::test_TABs_PARSE */
::routine main public
    TAB  ="09"x
    TAB2 =TAB||TAB

    PLANK=" "
    PLANK2=PLANK||PLANK

    str=plank2 "This" plank "is" plank "a" plank2 "test." || plank2
    word5exp=PLANK

    parse var str word1 word2 word3 word4 .
    self~assertSame("This", word1)
    self~assertSame("is", word2)
    self~assertSame("a", word3)
    self~assertSame("test.", word4)

    parse var str word1 word2 word3 word4 word5 .
    self~assertSame("This", word1)
    self~assertSame("is", word2)
    self~assertSame("a", word3)
    self~assertSame("test.", word4)
    self~assertSame("", word5)

    parse var str word1 word2 word3 word4 word5
    self~assertSame("This", word1)
    self~assertSame("is", word2)
    self~assertSame("a", word3)
    self~assertSame("test.", word4)
    self~assertSame(word5exp, word5)


    str=plank2 "This" plank "is" tab "a" plank2 "test."   || tab2
    word5exp=TAB

    parse var str word1 word2 word3 word4 .
    self~assertSame("This", word1)
    self~assertSame("is", word2)
    self~assertSame("a", word3)
    self~assertSame("test.", word4)

    parse var str word1 word2 word3 word4 word5 .
    self~assertSame("This", word1)
    self~assertSame("is", word2)
    self~assertSame("a", word3)
    self~assertSame("test.", word4)
    self~assertSame("", word5)

    parse var str word1 word2 word3 word4 word5
    self~assertSame("This", word1)
    self~assertSame("is", word2)
    self~assertSame("a", word3)
    self~assertSame("test.", word4)
    self~assertSame(word5exp, word5)


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
