/* extracted from socketClass::test_options_timeout */
::routine main public
  s = .socket~new()

  milliseconds = 300
  self~assertSame(0, s~setOption("SO_RCVTIMEO", milliseconds))
  -- there seems to be some rounding/adjusting going on on Linux
  -- we accept a delta of up to 10 ms
  ms = s~getOption("SO_RCVTIMEO")
  self~assertTrue((milliseconds - ms)~abs <= 10, "SO_RCVTIMEO set to" milliseconds", get returns" ms)

  milliseconds = 9876
  self~assertSame(0, s~setOption("SO_SNDTIMEO", milliseconds))
  -- there seems to be some rounding/adjusting going on on Linux
  -- we accept a delta of up to 10 ms
  ms = s~getOption("SO_SNDTIMEO")
  self~assertTrue((milliseconds - ms)~abs <= 10, "SO_SNDTIMEO set to" milliseconds", get returns" ms)
  self~assertSame(0, s~close)

-- SO_RCVBUF, SO_SNDBUF
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
