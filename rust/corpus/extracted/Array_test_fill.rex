/* extracted from Array::test_fill */
::routine main public
  arr = .array~new
  -- zero size array doesn't fill any thing
  arr~fill(0)
  self~assertSame(arr~size, 0)
  self~assertSame(arr~items, 0)
  -- fill empty sized array
  arr = .array~new(3)
  arr~fill(3)
  self~assertSame(arr~size, 3)
  self~assertSame(arr~items, 3)
  self~assertTrue(sameContent(arr, .array~of(3, 3, 3)))
  -- arrays keep track of the last item for the append method
  -- to function efficiently.  Make sure this still works
  arr~append(4)
  self~assertSame(arr~size, 4)
  self~assertSame(arr~items, 4)
  self~assertTrue(sameContent(arr, .array~of(3, 3, 3, 4)))

  arr = .array~of(1,,,4)
  -- fill a sparse array
  arr~fill('a')
  self~assertSame(arr~size, 4)
  self~assertSame(arr~items, 4)
  self~assertTrue(sameContent(arr, .array~of("a", "a", "a", "a")))

  arr = .array~new(2, 2)
  arr~fill(1)
  self~assertSame(arr~size, 4)
  self~assertSame(arr~items, 4)
  loop i = 1 to 2
     loop j = 1 to 2
        self~assertSame(arr[i, j], 1)
     end
  end

  arr = .array~new(2, 2, 2)
  arr~fill(1)
  self~assertSame(arr~size, 8)
  self~assertSame(arr~items, 8)
  loop i = 1 to 2
     loop j = 1 to 2
        loop k = 1 to 2
           self~assertSame(arr[i, j, k], 1)
        end
     end
  end


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
