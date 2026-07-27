/* extracted from OPTIONS::test_options_trace_all_with_floating_method */
::routine main public
  -- we redirect 'trace all' output to an Array
  trace = .Array~new
  .error~destination(.ArrayStream~new(trace))

  r = .Routine~new("", .Array~of(  -
    '.c~r(.methods["FLOAT"])    ', -
    '::method float             ', -
    '::class c                  ', -
    '::method r class           ', -
    '    forward message "run"  ', -
    '::options trace "a"        '))
  r~call

bShow=.false         -- set to .true to display actual trace
if bShow=.true then  -- display trace
do
   do counter c1 traceObj over trace
      say .line "#" c1~right(2)":" traceObj~makeString
   end
end
  /* this is the expected trace output:
617 #  1:        >I> Routine "" in package "".
617 #  2:        <I< Routine "" in package "".
617 #  3:        >I> Routine "" in package "".
617 #  4:      1 *-* .c~r(.methods["FLOAT"])
617 #  5:        >I> Method "R" with scope "C" in package "".
617 #  6:      5 *-* forward message "run"
617 #  7:        >I> Method "*UNNAMED*" with scope ".NIL" in package "".
617 #  8:        <I< Method "*UNNAMED*" with scope ".NIL" in package "".
617 #  9:        <I< Method "R" with scope "C" in package "".
617 # 10:        <I< Method "R" with scope "C" in package "".
617 # 11:        <I< Routine "" in package "".
  */
  -- we're not expected to crash (that's what the bug report is about)
  -- but since we already collect the trace ouput, we can as well
  -- check a few things
  do idx over 7, 8   -- test the 7th (>I>) and the 8th (<I<) entry
     -- the floating method's (missing) name is displayed as *UNNAMED*
     self~assertTrue(trace[idx]~makeString~contains("*UNNAMED*"))
     -- the floating method's scope is displayed as ".NIL"
     self~assertTrue(trace[idx]~makeString~contains('".NIL"'))
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
