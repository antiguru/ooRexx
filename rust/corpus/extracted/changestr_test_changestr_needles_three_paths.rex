/* extracted from changestr::test_changestr_needles_three_paths */
::routine main public
  str = "birdbird1bird"
  needle = "bird"
  do newNeedle over "", "cat", "fish", "mouse"
    do string over str, str || "_", "_" || str, "_" || str || "_"
      m = .MutableBuffer~new(string)
      -- use the changeStr() BIF (with a very different implementation) as reference
      self~assertSame(changeStr(needle, string, newNeedle), m~changeStr(needle, newNeedle))
      do count = 0 to 4
        m = .MutableBuffer~new(string)
        self~assertSame(changeStr(needle, string, newNeedle, count), m~changeStr(needle, newNeedle, count))
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
