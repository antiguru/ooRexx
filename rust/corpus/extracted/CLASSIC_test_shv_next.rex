/* extracted from CLASSIC::test_shv_next */
::routine main public
  shv = .Array~new(9)
  do shv~size
    shv~append(.Shv~new("N", ""))
  end

  -- we now have four variables defined: SELF, SUPER, SHV, and RESULT
  -- we have n+1 SHVBLOCKs and the last one must be LVAR
  self~assertSame(5, TestFNVariablePool(shv))
  self~assertSame(.Shv~LVAR, shv[5]~shvret) -- last+1 SHVBLOCK must be LVAR
  mustHave = "SELF", "SUPER", "SHV", "RESULT"
  do i = 1 to mustHave~items
    self~assertSame(.Shv~OK, shv[i]~shvret)
    self~assertTrue(mustHave~hasItem(shv[i]~shvname), shv[i]~shvname)
    mustHave~removeItem(shv[i]~shvname)
    select case shv[i]~shvname
      when "SHV"    then self~assertSame("an Array", shv[i]~shvvalue)
      when "RESULT" then self~assertSame(9, shv[i]~shvvalue)
      otherwise nop
    end
  end

  -- call a procedure where not a single variable is defined
  self~assertSame(1, procedureVariablePool(shv))
  self~assertSame(.Shv~LVAR, shv[1]~shvret) -- last+1 SHVBLOCK must be LVAR
  return

  procedureVariablePool: procedure
  return TestFNVariablePool(arg(1))


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
