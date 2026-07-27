/* extracted from SelectCase::test_10 */
::routine main public
  -- testing objects that override "=="

  caseObj = .testEquals~new("ABC")

  whenObj1 = .testEquals~new("DEF")
  whenObj2 = .testEquals~new("ABC")

   select case caseObj
      when whenObj1 then do
         match = 1
      end
      -- this one should match
      when whenObj2 then do
         match = 2
      end
      otherwise
         match = 3
   end

   self~assertSame(2, match)

   select case caseObj
      when whenObj1 then do
         match = 1
      end
      -- this one should match
      when whenObj1, whenObj2 then do
         match = 2
      end
      otherwise
         match = 3
   end

   self~assertSame(2, match)

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
