/* extracted from whiteSpace::test_TABs_DELWORD */
::routine main public
    TAB  ="09"x
    TAB2 =TAB||TAB

    PLANK=" "
    PLANK2=PLANK||PLANK

      -- test DELWORD
    s0  ="hey"
    s0_p="hey you"
    s0_p2="hey  you"
    s0_t="hey"TAB"you"
    s0_t2="hey"TAB2"you"

    s1="hey"PLANK"is-this"PLANK2"you"
    s2="hey"TAB"is-this"PLANK2"you"
    s3="hey"PLANK"is-this"TAB"you"
    s4="hey"PLANK2"is-this"TAB2"you"
    s5="hey"TAB"is-this"TAB2"you"
    s6="hey"TAB2"is-this"TAB2"you"

      -- delete second word
    self~assertSame(s0_p, delword(s1, 2, 1))
    self~assertSame(s0_t, delword(s2, 2, 1))
    self~assertSame(s0_p, delword(s3, 2, 1))
    self~assertSame(s0_p2, delword(s4, 2, 1))
    self~assertSame(s0_t, delword(s5, 2, 1))
    self~assertSame(s0_t2, delword(s6, 2, 1))

      -- delete everything starting and including the second word
    self~assertSame(s0||PLANK, delword(s1, 2))
    self~assertSame(s0||TAB, delword(s2, 2))
    self~assertSame(s0||PLANK, delword(s3, 2))
    self~assertSame(s0||PLANK2, delword(s4, 2))
    self~assertSame(s0||TAB, delword(s5, 2))
    self~assertSame(s0||TAB2, delword(s6, 2))


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
