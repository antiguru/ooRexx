/* extracted from Ticker::test_ticker_two_args_string_trigger */
::routine main public
  t1 = .tickerWaiter~new
  ticker = .Ticker~new(0.5, t1)
  self~assertTrue(t1~triggerCount = 0) -- shouldn't have triggered yet
  call syssleep 0.75                   -- let Ticker trigger once
  self~assertTrue(t1~triggerCount = 1, "should have triggered once, but triggered" t1~triggerCount "times")
  self~assertSame(ticker, t1~triggerTicker)
  self~assertSame(.nil, t1~triggerAttached)
  ticker~cancel
  self~assertTrue(t1~stopped)
  self~assertSame(ticker, t1~stoppedTicker)
  self~assertSame(.nil, t1~stoppedAttached)

-- same two tests as above, but with TimeSpan instead of string
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
