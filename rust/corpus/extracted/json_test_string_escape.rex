/* extracted from json::test_string_escape */
::routine main public
  j = .Json~new

  -- All code points may be placed within the quotation marks except for
  -- quote, backslash, and the control characters '00'x through '1f'x,
  -- which must be escaped.  There are eight two-character escape
  -- sequence representations \" \\ \/ \b \f \n \r \t.

  -- for control characters without two-character escapes \u00XX is used
  do c over xrange('00'x, '07'x, '0b'x, '0b'x, '0e'x, '1f'x)~makeArray("")
    self~assertSame(c, j~fromJson('"\u00' || c~c2x ||'"'))
    self~assertSame('"\u00' || c~c2x ||'"', j~toJson(c))
  end

  self~assertSame('"', j~fromJson('"\""'))
  self~assertSame("\", j~fromJson('"\\"'))
  self~assertSame("/", j~fromJson('"\/"'))
  self~assertSame('08'x, j~fromJson('"\b"'))
  self~assertSame('0c'x, j~fromJson('"\f"'))
  self~assertSame('0a'x, j~fromJson('"\n"'))
  self~assertSame('0d'x, j~fromJson('"\r"'))
  self~assertSame('09'x, j~fromJson('"\t"'))

  self~assertSame('"\""', j~toJson('"'))
  self~assertSame('"\\"', j~toJson("\"))
  self~assertSame('"\/"', j~toJson("/"))
  self~assertSame('"\b"', j~toJson('08'x))
  self~assertSame('"\f"', j~toJson('0c'x))
  self~assertSame('"\n"', j~toJson('0a'x))
  self~assertSame('"\r"', j~toJson('0d'x))
  self~assertSame('"\t"', j~toJson('09'x))

  -- generally speaking, \uXXXX escape sequences are unsupported as
  -- ooRexx doesn't provide Unicode support
  -- but instead of failing the parse, we just just keep any \uXXXX
  -- as-is for both the fromJson and the toJson methods
  -- fromJson doesn't un-escape, and toJson doesn't escape any \uXXXX
  escape = "abc\u123456"
  self~assertSame(escape, j~fromJson('"' || escape || '"'))
  self~assertSame('"' || escape || '"', j~toJson(escape))


-- array

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
