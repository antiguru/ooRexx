/* extracted from TRACE::test_trace_?_option */
::routine main public
  t = .TraceOutput~destination(.ArrayStream~new) -- capture .traceOutput
  .DebugInput~destination(.ArrayStream~new(5)~fill("trace 9")) -- belt and braces

  trace ?c
  self~assertSame("?C", trace())
  call trace "?" -- stop interactive debugging mode

  trace ?err
  self~assertSame("?E", trace())
  call trace "?"

  trace ?F
  self~assertSame("?F", trace())
  call trace "?"

  trace ???lbl -- triple ??? should be the same as a single ?
  self~assertSame("?L", trace())
  call trace "?"

  trace ?n
  self~assertSame("?N", trace())
  call trace "off" -- stop all tracing

  self~assertSame(0, t~items, "unexpected TRACE ?x output" t) -- expect no output

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
