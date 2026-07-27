/* extracted from LOOP::test_LOOP_counter */
::routine main public

  loop counter i k = 1 to 3
  end

  self~assertEquals(3, i)
  self~assertEquals(4, k)

  loop counter i k = 3 to 1
  end

  self~assertEquals(0, i)
  self~assertEquals(3, k)

  loop label abc counter i k = 1 to 3
  end abc

  self~assertEquals(3, i)
  self~assertEquals(4, k)

  loop counter i label abc k = 3 to 1
  end abc

  self~assertEquals(0, i)
  self~assertEquals(3, k)

  loop counter = 1 to 3
  end

  self~assertEquals(4, counter)

  -- uses the counter variable in the condition
  loop counter i while i < 3
  end

  self~assertEquals(3, i)

  -- uses the counter variable in the condition
  loop counter i until i > 3
  end

  self~assertEquals(4, i)

  -- uses the counter variable in the condition
  loop counter i while .false
  end

  self~assertEquals(0, i)

  -- uses the counter variable in the condition
  loop counter i until .true
  end

  self~assertEquals(1, i)

  loop counter i 3
  end

  self~assertEquals(3, i)

  loop counter i item over (1,2,3)
  end

  self~assertEquals(3, i)
  self~assertEquals(3, item)

  loop counter i with item item index index over (4,5,6)
  end

  self~assertEquals(3, i)
  self~assertEquals(3, index)
  self~assertEquals(6, item)

  loop counter i forever while i < 3
  end

  self~assertEquals(3, i)

  loop counter i forever until i > 3
  end

  self~assertEquals(4, i)

  loop counter i forever while .false
  end

  self~assertEquals(0, i)

  loop counter i forever until .true
  end

  self~assertEquals(1, i)

  -- a simple loop is the same as DO FOREVER
  loop counter i
     if i == 3 then leave
  end

  self~assertEquals(3, i)

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
