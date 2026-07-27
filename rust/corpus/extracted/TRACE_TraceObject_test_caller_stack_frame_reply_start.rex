/* extracted from TRACE_TraceObject::test_caller_stack_frame_reply_start */
::routine main public
-- test callerStackFrame correct

traceLog=.array~new
.traceObject~option='P'
.traceObject~collector=traceLog

callerName="testCallerStackFrame_ReplyStart"
r=.routine~new(callerName, .resources~test_caller_stackframe_reply_start_program)
r~call         -- note: this is a method call to an object!
.traceObject~collector=.nil

/*
-- show actual traceLog
say "--->"
do counter c traceObj over traceLog
    say traceObj
end
say "<---"
*/

callerStackFrames=.list~new
-- now analyze traceLog, first check whether expected sequence
expectedTraceLog=.resources~test_caller_stackframe_reply_start_tracelog
do counter c traceObj over traceLog
   str=traceObj~makeString
   self~assertEquals(str, expectedTraceLog[c])
   if pos(">I>",str)>0 then
      callerStackFrames~append(traceObj)  -- save traceObject which needs to have a CALLERSTACKFRAME entry
end

--say "... checking callerStackFrames ..."
countThreadId_in_CSF=0
do csf over callerStackFrames
   self~assertTrue(csf~hasEntry("CALLERSTACKFRAME"))
   countThreadId_in_CSF+=csf~callerStackFrame~hasEntry("THREAD")
end

self~assertEquals(countThreadId_in_CSF,2)


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
