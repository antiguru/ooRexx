/* extracted from TRACE::test_trace_?a */
::routine main public
  t = .TraceOutput~destination(.ArrayStream~new) -- capture .traceOutput
  .DebugInput~destination(.ArrayStream~of("trace 99")) -- continue tracing

  trace ?a
  self~assertSame("?A", trace())
  call trace "off" -- stop all tracing

  /* this is the expected TRACE ?A output
       +++ "pppppppp METHOD path/to/ooRexx/base/keyword/TRACE.testGroup"
   nnn *-* self~assertSame("?A", trace())
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
   nnn *-*   call trace "off" -- stop all tracing
  */
  self~assertSame(4, t~items, "unexpected TRACE ?A output" t)
  self~assertSame("+++ *-* +++ Interactive *-*", -
   t[1]~word(1) t[2]~word(2) t[3]~subWord(1, 2) t[4]~word(2), -
   "unexpected TRACE ?A output" t)

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
