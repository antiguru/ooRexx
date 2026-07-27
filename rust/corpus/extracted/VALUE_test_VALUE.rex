/* extracted from VALUE::test_VALUE */
::routine main public
    /* After: Drop A3; A33=7; K=3; fred='K'; list.5='Hi' */
    Drop A3; A33=7; K=3; fred='K'; list.5='Hi'

    self~assertSame('A3', VALUE('a'k))  /* looks up A3 */
    self~assertSame('7', VALUE('a'k||k))
    self~assertSame('K', VALUE('fred'))  /* looks up FRED */
    self~assertSame('3', VALUE(fred))  /* looks up K */
    self~assertSame('3', VALUE(fred,5))  /* looks up K and then sets K=5 */
    self~assertSame('5', VALUE(fred))  /* looks up K */
    self~assertSame('Hi', VALUE('LIST.'k))  /* looks up LIST.5 */

   --
    /* Given that an external variable FRED has a value of 4 */
    share = 'ENVIRONMENT'
    call value 'FRED', 4, share  -- set FRED to 4
    self~assertEquals(4, VALUE('FRED',7,share))/* says '4' and assigns */
                                                             /* FRED a new value of 7 */
    self~assertEquals(7, VALUE('FRED', ,share)) /* says '7' */

    call value 'FRED', .nil, share  -- delete 'FRED' from environment
    self~assertSame("", VALUE('FRED', ,share))

   -- test set/getting .environment-entries
    call value 'UhU', 1, ''         -- define entry 'UHU' in .environment
    self~assertSame(1, VALUE('UhU',   , ''))
    call value 'UhU', .nil, ''      -- set entry in .environment to .nil
    self~assertSame(.nil, VALUE('UhU', , ''))
    .environment~remove('UHU')      -- delete entry in .environment
    self~assertSame('.UHU', VALUE('UhU', , ''))





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
