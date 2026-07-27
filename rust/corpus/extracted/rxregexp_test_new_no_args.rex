/* extracted from rxregexp::test_new_no_args */
::routine main public
  p = .RegularExpression~new
  -- should default to "maximal")
  self~assertEquals(0, p~parse("ab+"))
  self~assertEquals(1, p~pos("abbbbb"))
  self~assertEquals(6, p~position) -- must have matched whole string

  -- keep default by specifying "current")
  self~assertEquals(0, p~parse("ab+", "CURRENT"))
  self~assertEquals(1, p~pos("abbbbb"))
  self~assertEquals(6, p~position) -- must have matched whole string

  -- override default by specifying "minimal")
  self~assertEquals(0, p~parse("ab+", "minimal"))
  self~assertEquals(1, p~pos("abbbbb"))
  self~assertEquals(2, p~position) -- should have just matched "ab"

-- new() will accept omitted template, but will fail with null template
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
