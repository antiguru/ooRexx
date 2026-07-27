/* extracted from TRACE::test_trace_?r */
::routine main public
  t = .TraceOutput~destination(.ArrayStream~new) -- capture .traceOutput
  .DebugInput~destination(.ArrayStream~of("trace 99")) -- continue tracing

  trace ?r
  self~assertSame("?R", trace())
  call trace "off" -- stop all tracing

  -- with no expressions, the expected TRACE ?R output is the same as
  -- above TRACE ?A output
  self~assertSame(4, t~items, "unexpected TRACE ?R output" t)
  self~assertSame("+++ *-* +++ Interactive *-*", -
   t[1]~word(1) t[2]~word(2) t[3]~subWord(1, 2) t[4]~word(2), -
   "unexpected TRACE ?R output" t)

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
