/* extracted from Class::test_METHODS */
::routine main public
   a_o =getBags(.Object~methods(.nil))       -- get only Object methods
   a_v =getBags(.Vehicle~methods(.nil))      -- get only Vehicle methods
   a_wv=getBags(.WaterVehicle~methods(.nil)) -- get only WaterVehicle methods
   a_wv_all=getBags(.WaterVehicle~methods)   -- get all methods

   self~assertTrue(a_o[1]~Subset(a_wv_all[1]))
   self~assertTrue(a_o[2]~Subset(a_wv_all[2]))

   self~assertTrue(a_v[1]~Subset(a_wv_all[1]))
   self~assertTrue(a_v[2]~Subset(a_wv_all[2]))

   self~assertTrue(a_wv[1]~Subset(a_wv_all[1]))
   self~assertTrue(a_wv[2]~Subset(a_wv_all[2]))

   do i=1 to 2
      tmp=.bag~new~union(a_o[i])~union(a_v[i])~union(a_wv[i])
      self~assertTrue(tmp~subset(a_wv_all[i]))
      self~assertTrue(a_wv_all[i]~subset(tmp))
   end

   self~assertEquals(a_wv_all[1]~items, a_o[1]~items + a_v[1]~items + a_wv[1]~items)

   return

getBags: procedure         -- return indices in an index bag, and method items in a method bag
   use arg s
   a=.array~new

   if s~available then
   do
      a[1]=.bag~new    -- index
      a[2]=.bag~new    -- item (object)
   end

   do while s~available
      a[1]~put(s~index)
      a[2]~put(s~item)
      s~next
   end
   return a



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
