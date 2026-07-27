/* extracted from ERRORTEXT::test_ERRORTEXT */
::routine main public
    self~assertSame('Label not found.', ERRORTEXT(16))
    self~assertSame("", ERRORTEXT(60))

    self~assertSame('', ERRORTEXT(1))
    self~assertSame('', ERRORTEXT(2))
    self~assertSame('Failure during initialization.', ERRORTEXT(3))
    self~assertSame('Program interrupted.', ERRORTEXT(4))
    self~assertSame('System resources exhausted.', ERRORTEXT(5))
    self~assertSame('Unmatched "/*" or quote.', ERRORTEXT(6))
    self~assertSame('WHEN or OTHERWISE expected.', ERRORTEXT(7))
    self~assertSame('Unexpected THEN or ELSE.', ERRORTEXT(8))
    self~assertSame('Unexpected WHEN or OTHERWISE.', ERRORTEXT(9))
    self~assertSame('Unexpected or unmatched END.', ERRORTEXT(10))
    self~assertSame('Control stack full.', ERRORTEXT(11))
    self~assertSame('', ERRORTEXT(12))
    self~assertSame('Invalid character in program.', ERRORTEXT(13))
    self~assertSame('Incomplete DO/LOOP/SELECT/IF.', ERRORTEXT(14))
    self~assertSame('Invalid hexadecimal or binary string.', ERRORTEXT(15))
    self~assertSame('Label not found.', ERRORTEXT(16))
    self~assertSame('Unexpected PROCEDURE.', ERRORTEXT(17))
    self~assertSame('THEN expected.', ERRORTEXT(18))
    self~assertSame('String or symbol expected.', ERRORTEXT(19))
    self~assertSame('Symbol expected.', ERRORTEXT(20))
    self~assertSame('Invalid data on end of clause.', ERRORTEXT(21))
    self~assertSame('Invalid character string.', ERRORTEXT(22))
    self~assertSame('Invalid data string.', ERRORTEXT(23))
    self~assertSame('Invalid TRACE request.', ERRORTEXT(24))
    self~assertSame('Invalid subkeyword found.', ERRORTEXT(25))
    self~assertSame('Invalid whole number.', ERRORTEXT(26))
    self~assertSame('Invalid DO or LOOP syntax.', ERRORTEXT(27))
    self~assertSame('Invalid LEAVE or ITERATE.', ERRORTEXT(28))
    self~assertSame('Environment name too long.', ERRORTEXT(29))
    self~assertSame('Name or symbol too long.', ERRORTEXT(30))
    self~assertSame('Name starts with number or ".".', ERRORTEXT(31))
    self~assertSame('', ERRORTEXT(32))
    self~assertSame('Invalid expression result.', ERRORTEXT(33))
    self~assertSame('Logical value not 0 or 1.', ERRORTEXT(34))
    self~assertSame('Invalid expression.', ERRORTEXT(35))
    self~assertSame('Unmatched "(" or "[" in expression.', ERRORTEXT(36))
    self~assertSame('Unexpected ",", ")", or "]".', ERRORTEXT(37))
    self~assertSame('Invalid template or pattern.', ERRORTEXT(38))
    self~assertSame('Evaluation stack overflow.', ERRORTEXT(39))
    self~assertSame('Incorrect call to routine.', ERRORTEXT(40))
    self~assertSame('Bad arithmetic conversion.', ERRORTEXT(41))
    self~assertSame('Arithmetic overflow/underflow.', ERRORTEXT(42))
    self~assertSame('Routine not found.', ERRORTEXT(43))
    self~assertSame('Function or message did not return data.', ERRORTEXT(44))
    self~assertSame('No data specified on function RETURN.', ERRORTEXT(45))
    self~assertSame('Invalid variable reference.', ERRORTEXT(46))
    self~assertSame('Unexpected label.', ERRORTEXT(47))
    self~assertSame('Failure in system service.', ERRORTEXT(48))
    self~assertSame('Interpretation error.', ERRORTEXT(49))
    self~assertSame('', ERRORTEXT(50))
    self~assertSame('', ERRORTEXT(51))
    self~assertSame('', ERRORTEXT(52))
    self~assertSame('', ERRORTEXT(53))
    self~assertSame('', ERRORTEXT(54))
    self~assertSame('', ERRORTEXT(55))
    self~assertSame('', ERRORTEXT(56))
    self~assertSame('', ERRORTEXT(57))
    self~assertSame('', ERRORTEXT(58))
    self~assertSame('', ERRORTEXT(59))
    self~assertSame('', ERRORTEXT(60))
    self~assertSame('', ERRORTEXT(61))
    self~assertSame('', ERRORTEXT(62))
    self~assertSame('', ERRORTEXT(63))
    self~assertSame('', ERRORTEXT(64))
    self~assertSame('', ERRORTEXT(65))
    self~assertSame('', ERRORTEXT(66))
    self~assertSame('', ERRORTEXT(67))
    self~assertSame('', ERRORTEXT(68))
    self~assertSame('', ERRORTEXT(69))
    self~assertSame('', ERRORTEXT(70))
    self~assertSame('', ERRORTEXT(71))
    self~assertSame('', ERRORTEXT(72))
    self~assertSame('', ERRORTEXT(73))
    self~assertSame('', ERRORTEXT(74))
    self~assertSame('', ERRORTEXT(75))
    self~assertSame('', ERRORTEXT(76))
    self~assertSame('', ERRORTEXT(77))
    self~assertSame('', ERRORTEXT(78))
    self~assertSame('', ERRORTEXT(79))
    self~assertSame('', ERRORTEXT(80))
    self~assertSame('', ERRORTEXT(81))
    self~assertSame('', ERRORTEXT(82))
    self~assertSame('', ERRORTEXT(83))
    self~assertSame('', ERRORTEXT(84))
    self~assertSame('', ERRORTEXT(85))
    self~assertSame('', ERRORTEXT(86))
    self~assertSame('', ERRORTEXT(87))
    self~assertSame('Invalid argument.', ERRORTEXT(88))
    self~assertSame('Variable or message term expected.', ERRORTEXT(89))
    self~assertSame('External name not found.', ERRORTEXT(90))
    self~assertSame('No result object.', ERRORTEXT(91))
    self~assertSame('OLE error.', ERRORTEXT(92))
    self~assertSame('Incorrect call to method.', ERRORTEXT(93))
    self~assertSame('', ERRORTEXT(94))
    self~assertSame('', ERRORTEXT(95))
    self~assertSame('', ERRORTEXT(96))
    self~assertSame('Object method not found.', ERRORTEXT(97))
    self~assertSame('Execution error.', ERRORTEXT(98))
    self~assertSame('Translation error.', ERRORTEXT(99))

    self~assertTrue(test100())
    self~assertTrue(test101())

    return

    ----------------------------------------------------------------------

    test100: procedure  -- nonexisting error text number
       signal on syntax name test100syntax
       call errortext(100) -- should invoke a syntax error
       return .false       -- arriving here is an error!
    test100syntax:         -- syntax error # 40.903
       return condition("o")~code=40.903

    ----------------------------------------------------------------------

    test101: procedure  -- not using a whole number as index
       signal on syntax name test101syntax
       a='blabla';
       errortext(a)        -- should invoke a syntax error
       return .false
    test101syntax:         -- syntax error # 40.12
       return condition("o")~code=40.12


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
