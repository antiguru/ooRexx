/* extracted from RxMath::test_infinity */
::routine main public
    self~assertSame("+infinity", RxCalcExp(709.8), "exp(709.8) should be +inf:" RxCalcExp(709.8))

    -- maximum double is ~1.79769e308; log(1.79769e308) is ~709.78
    self~assertTrue(RxCalcExp(709.7) > 1e308, "exp(709.7) should work:" RxCalcExp(709.7))
    self~assertSame("+infinity", RxCalcExp(709.8), "exp(709.8) should be +inf:" RxCalcExp(709.8))

    self~assertSame("-infinity", RxCalcLog(0))
    self~assertSame("-infinity", RxCalcLog10(0))

    -- 2^1023 < maximum double < 2^1024
    self~assertTrue(RxCalcPower(2, 1023) > 8e307, "power(2, 1023) should work:" RxCalcPower(2, 1023))
    self~assertSame("+infinity", RxCalcPower(2, 1024), "power(2, 1024) should be +inf:" RxCalcPower(2, 1024))

    -- (-2)^1023 > minimum double > (-2)^1025
    self~assertTrue(RxCalcPower(-2, 1023) < -1e307, "power(-2, 1023) should work:" RxCalcPower(-2, 1023))
    self~assertSame("+infinity", RxCalcPower(-2, 1024), "power(-2, 1024) should be +inf:" RxCalcPower(-2, 1024))
    self~assertSame("-infinity", RxCalcPower(-2, 1025), "power(-2, 1025) should be -inf:" RxCalcPower(-2, 1025))

    -- a bit arbitrary, but well ..
    self~assertSame("+infinity", RxCalcTan(90))
    self~assertSame("-infinity", RxCalcTan(270))


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
