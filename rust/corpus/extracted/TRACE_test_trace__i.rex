/* extracted from TRACE::test_trace_?i */
::routine main public
  t = .TraceOutput~destination(.ArrayStream~new) -- capture .traceOutput
  .DebugInput~destination(.ArrayStream~of("trace 999")) -- continue tracing

  trace ?i
  self~assertSame("?I", trace())
  call trace "off" -- stop all tracing

  /* this is the expected TRACE ?I output
       +++ "pppppppp METHOD path/to/ooRexx/base/keyword/TRACE.testGroup"
   nnn *-* self~assertSame("?I", trace())
       >V>   SELF => "a TRACE.TESTGROUP"
       >L>   "?I"
       >A>   "?I"
       >F>   TRACE => "?I"
       >A>   "?I"
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
   nnn *-*   call trace "off" -- stop all tracing
       >L>     "off"
       >A>     "off"
  */
  self~assertSame(11, t~items, "unexpected TRACE ?I output" t)
  self~assertSame("+++ *-* >V> >L> >A> >F> >A> +++ Interactive *-* >L> >A>", -
   t[1]~word(1) t[2]~word(2) t[3]~word(1) t[4]~word(1) t[5]~word(1) t[6]~word(1) -
   t[7]~word(1) t[8]~subWord(1, 2) t[9]~word(2) t[10]~word(1) t[11]~word(1), -
   "unexpected TRACE ?I output" t)

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
