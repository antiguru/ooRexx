/* extracted from format::test_format_bug_1474_full */
::routine main public
  signal off lostdigits -- n has more digits than the current precision
  do case = 1 to 3
    select
      when case = 1 then do
        n = 12345678901234
        reps = 30
      end
      when case = 2 then do
        numeric digits 18
        n = 1234567890123456789012345
        reps = 60
      end
      when case = 3 then do
        numeric digits 19
        n = 1234567890123456789012345
      end
    end

    do reps
      f = n~format(, 5)
      parse var f before "." after "E"
      self~assertTrue(before~dataType("number"), "before must be all-numeric:" f)
      self~assertTrue(after~dataType("number"), "after must be all-numeric:" f)
      self~assertSame(5, after~length, "after must be length 5:" f)
      n = n / 10
    end
  end

-- tests for the [bugs:1695] "if format(99.999,,2,,2) \== '1.00E+2' then say 'ok' segfaults"

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
