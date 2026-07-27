/* extracted from TRACE_TraceObject::test_object_and_scope */
::routine main public
   -- .routine~new("myTestRoutine",.resources~myObjectScopeTestRoutine)~~call; say "<--"
   .traceObject~option='P'
   arr=.array~new
   .traceObject~collector=arr
      -- load and run the code
   .routine~new("myTestRoutine",.resources~myObjectScopeTestRoutine)~~call
   .traceObject~collector=.nil

   -- check expected tracelines
   self~assertEquals(arr~items, 19)
/* --
say "arr~items:" arr~items":"
do counter c i over arr
   say c~right(2)":" i~traceline
   tmp=i~stackframe~target    -- if target has a value, we are in a method
   scope=.nil
   if \tmp~isNil then         -- if target has a value, we are in a method
       scope=i~stackframe~executable~scope   -- get method's scope
   say pp(tmp~string) pp(scope~string) "|" pp(i)
   say
end
say "---"
--*/
   -- get expected trace lines
   traces=.resources~myObjectScopeTestRoutineTrace
   do counter c1 traceObj over arr
      actual    =traceObj~traceline -- get traceLine entry
      predefined=traces[c1]         -- get expected traceLine
      self~assertSame(actual,predefined,"traceObject #" c1": TRACELINE entry"  pp(actual) "does not match expected traceline:" pp(predefined))
   end
   -- expected            1     2   3   4   5   6     7     8     9    10    11    12   13     14  15  16  17  18   19
   objectids=.array~of(.nil, .nil, "", "", "", "", .nil, .nil,   "",   "",   "",   "", .nil, .nil, "", "", "", "", .nil)
   scopes   =.array~of(.nil, .nil, "", "", "", "", .nil, .nil, .nil, .nil, .nil, .nil, .nil, .nil, "", "", "", "", .nil)

   -- in actual trace
   do i=1 to arr~items
      stackFr   =arr[i]~stackFrame
      type      =stackFr~type
      obj       =stackFr~target
      if type="METHOD" then scope=stackFr~executable~scope
                       else scope=.nil
      self~assertSame(objectids[i]~isNil, obj  ~isNil, "TraceObject #" i "OBJECT" obj  ~string)
      self~assertSame(   scopes[i]~isNil, scope~isNil, "TraceObject #" i "SCOPE"  scope~string)
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
