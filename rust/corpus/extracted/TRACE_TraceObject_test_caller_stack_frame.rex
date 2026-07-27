/* extracted from TRACE_TraceObject::test_caller_stack_frame */
::routine main public
-- test callerStackFrame correct

traceLog=.array~new
.traceObject~option='P'
.traceObject~collector=traceLog

callerName="testCallerStackFrame"
r=.routine~new(callerName, .resources~test_caller_stackframe)
do_the_work_stack_frame=r~call   -- note: this is a method call to an object!
.traceObject~collector=.nil

callerStackFrame=.nil
do traceObj over traceLog  -- find invocation entry traceObject for routine "OHA"
   if traceObj~traceLine~pos('>I> Routine "OHA"')>0 then
   do
       ohaCallerStackFrame=traceObj~callerStackFrame
       leave
   end
end

indices="EXECUTABLE", "LINE", "NAME", "TARGET", "TYPE"
call compareStackFrames ohaCallerStackFrame, do_the_work_stack_frame, indices, self

--> assert arguments~items the same!
self~assertEquals(ohaCallerStackFrame~arguments~items, 0, "expecting 0 items")
self~assertEquals(ohaCallerStackFrame~arguments~items, do_the_work_stack_frame~arguments~items, "arguments~items are different!")
self~assertSame(ohaCallerStackFrame~traceLine    , "     5 *-* call oha   -- both statements in the same line!")
self~assertSame(do_the_work_stack_frame~traceLine, "     5 *-* thisStackFrame=.context~stackframes[1];")
exit

   -- assert equality
compareStackFrames: procedure
   use arg ohaCallerStackFrame, do_the_work_stack_frame, indices, self
   -- do idx over ohaCallerStackFrame~allindexes
   do idx over indices -- ohaCallerStackFrame~allindexes
--      say "idx:" idx~left(16,'.') "ohaCallerStackFrame value:" ohaCallerStackFrame~send(idx)
--      say "                      do_the_work_stack_frame  :" do_the_work_stack_frame~send(idx)
      self~assertSame(ohaCallerStackFrame~send(idx), do_the_work_stack_frame~send(idx), "caller's stackFrame" idx": not the same!")
--    say
   end
   return

   -- attention: this test depends on the exact code of the following definition, do NOT edit!
/* ---> do NOT edit the following RESOURCE lines */
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
