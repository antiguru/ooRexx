/* extracted from Array::test_extend_multi_dimensional */
::routine main public
  a = .Array~new; a[2, 3] = "value"; a[2, 7] = "other"; self~assertSame("value", a[2, 3])
  a = .Array~new; a[2, 3] = "value"; a[5, 3] = "other"; self~assertSame("value", a[2, 3])
  a = .Array~new; a[2, 3] = "value"; a[5, 7] = "other"; self~assertSame("value", a[2, 3])

  a = .Array~new; a[2, 3, 5] = "value"; a[2,  3, 13] = "other"; self~assertSame("value", a[2, 3, 5])
  a = .Array~new; a[2, 3, 5] = "value"; a[2, 11,  5] = "other"; self~assertSame("value", a[2, 3, 5])
  a = .Array~new; a[2, 3, 5] = "value"; a[2, 11, 13] = "other"; self~assertSame("value", a[2, 3, 5])
  a = .Array~new; a[2, 3, 5] = "value"; a[7,  3,  5] = "other"; self~assertSame("value", a[2, 3, 5])
  a = .Array~new; a[2, 3, 5] = "value"; a[7,  3, 13] = "other"; self~assertSame("value", a[2, 3, 5])
  a = .Array~new; a[2, 3, 5] = "value"; a[7, 11,  5] = "other"; self~assertSame("value", a[2, 3, 5])
  a = .Array~new; a[2, 3, 5] = "value"; a[7, 11, 13] = "other"; self~assertSame("value", a[2, 3, 5])

  a = .Array~new; a[2, 3, 5, 7] = "value"; a[ 2,  3,  5, 19] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[ 2,  3, 17,  7] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[ 2,  3, 17, 19] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[ 2, 13,  5,  7] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[ 2, 13,  5, 19] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[ 2, 13, 17,  7] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[ 2, 13, 17, 19] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[11,  3,  5,  7] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[11,  3,  5, 19] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[11,  3, 17,  7] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[11,  3, 17, 19] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[11, 13,  5,  7] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[11, 13,  5, 19] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[11, 13, 17,  7] = "other"; self~assertSame("value", a[2, 3, 5, 7])
  a = .Array~new; a[2, 3, 5, 7] = "value"; a[11, 13, 17, 19] = "other"; self~assertSame("value", a[2, 3, 5, 7])


/* Test whether both collections contain the same entries.
   returns .true, if the same, .false else
*/
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
