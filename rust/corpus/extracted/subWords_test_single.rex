/* extracted from subWords::test_single */
::routine main public
  single = .Array~of(123)
  self~assertEquals(single, 123~subWords)
  self~assertEquals(single, 123~subWords(1))
  self~assertEquals(single, 123~subWords(, 1))
  self~assertEquals(single, 123~subWords(, 2))
  self~assertEquals(single, "123 two"~subWords(, 1))
  self~assertEquals(single, "123 two"~subWords(1, 1))
  self~assertEquals(single, " one 123"~subWords(2))
  self~assertEquals(single, (" one" || .String~tab || "123")~subWords(2, 1))
  self~assertEquals(single, " one  123  three "~subWords(2, 1))

  -- whitespace is just blank and tab, nothing else
  noWhiteSpace = xrange('00'x, '08'x, '0a'x, '1f'x) || (" "~c2d + 128)~d2c
  self~assertEquals(.Array~of(noWhitespace), noWhitespace~subWords)

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
