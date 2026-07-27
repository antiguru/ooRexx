/* extracted from RANDOM::testRandom02 */
::routine main public
  do 100
      r = random()
      self~assertTrue(r >= 0 & r<= 999)
  end

  r = random(,,12345)
  self~assertTrue(r >= 0 & r<= 999)

  r = random(1,1)
  self~assertSame(1, r)
  r = random(1,2)
  self~assertTrue(r >= 1 & r<= 2)

  r = random(5)
  self~assertTrue(r >= 0 & r<= 5)
  r = random(0)
  self~assertSame(0, r)
  r = random(999999998)
  self~assertTrue(r >= 0 & r<= 999999998)
  r = random(,0)
  self~assertSame(0, r)
  r = random(,1)
  self~assertTrue(r >= 0 & r<= 1)
  r = random(0,1,0)
  self~assertTrue(r >= 0 & r<= 1)

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
