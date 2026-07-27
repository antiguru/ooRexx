/* extracted from encode_decodeBase64::test_EncodeBase64_DecodeBase64 */
::routine main public
   allChars=xrange("00"x, "ff"x) -- create a string with all 8-Bit chars
   do i=1 to allChars~length
      a=allChars~left(i)         -- create a string from the left
      self~assertSame(a, a~encodeBase64~decodeBase64)  -- test whether same value
      -- self~assertEquals(a, a1~encodeBase64~decodeBase64)  -- test whether same value

      b=allChars~right(i)        -- create a string from the right
      self~assertSame(b, b~encodeBase64~decodeBase64)  -- test whether same value
   end

   a="Hello world"
   a1=a~encodeBase64          -- encode the string with ooRexx
      -- "Hello world" encoded value from Perl, Ruby, etc. according to Salvador's article
   a2="SGVsbG8gd29ybGQ="

   self~assertSame(a, a2~decodeBase64)
   self~assertSame(a, a1~decodeBase64)


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
