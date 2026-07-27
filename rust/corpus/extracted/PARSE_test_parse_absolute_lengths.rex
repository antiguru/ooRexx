/* extracted from PARSE::test_parse_absolute_lengths */
::routine main public
  -- > allows true length match and usage of 0 length

  --           123456789
  --             v
  --              v
  --                 v
  parse value "01A0002CC" with len1 +2 val1 >(len1) len2 +2 val2 >(len2)
  self~assertSame("01", len1)
  self~assertSame("A", val1)
  self~assertSame("00", len2)
  self~assertSame("", val2)


  -- < extract chars between match position and previous characters (parses "23" from the string)
  --              v
  parse value "1234567890" with "4" prefix <2
  self~assertSame("23", prefix)

  --            v
  parse value "1234567890" with "2" prefix <2
  self~assertSame("1", prefix)

  parse value "1234567890" with "234" prefix <2
  self~assertSame("1", prefix)

  --           v
  parse value "1234567890" with "1" prefix <2
  self~assertSame("", prefix)

  --           v
  parse value "1234567890" with "1" postfix >2
  self~assertSame("12", postfix)

  parse value "1234567890" with "12345" postfix >2
  self~assertSame("12", postfix)

  --                  v
  parse value "1234567890" with "8" postfix >2
  self~assertSame("89", postfix)

  parse value "1234567890" with "890" postfix >2
  self~assertSame("89", postfix)

  parse value "1234567890" with "890" postfix >0
  self~assertSame("", postfix)

  --                    v
  parse value "1234567890" with "0" postfix >2
  self~assertSame("0", postfix)

  --                    v
  parse value "1234567890" with "0" +1 postfix >2
  self~assertSame("", postfix)

  --                    v
  parse value "1234567890" with "0" postfix >0
  self~assertSame("", postfix)

  --                    v
  parse value "1234567890" with "0" +1 postfix >0
  self~assertSame("", postfix)



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
