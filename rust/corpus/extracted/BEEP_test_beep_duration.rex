/* extracted from BEEP::test_beep_duration */
::routine main public
  -- Except for Windows and Linux there is generally no frequency and
  -- duration support available.  And even on Linux this will typically
  -- require sudo.  So if we are on BSD or Darwin, or we have no sudo
  -- on Linux, BEEP will fall back to printing a BEL character which
  -- returns almost immediately, not honoring the given duration.
  call time "r"
  call beep 37, 10 -- 37 Hz should intentionally be inaudible
  duration = time("e") * 1000 / 1
  -- we expect BEEP to take either less than 2 ms (fall-back case) or
  -- around 10 ms (fully supported beep) to complete this request.
  -- we allow up to 35 ms as Windows seems to be rather inaccurate.
  self~assertTrue(duration < 2 | duration >= 10 & duration <= 35, "10 ms BEEP took" duration "ms")


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
