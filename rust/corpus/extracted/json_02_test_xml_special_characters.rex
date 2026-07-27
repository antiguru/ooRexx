/* extracted from json_02::test_xml_special_characters */
::routine main public
  json = .json~new
  dir = .directory~new
  dir["amp"] = "a&b"
  dir["lt"] = "a<b"
  dir["gt"] = "a>b"
  dir["quot"] = 'a"b'
  dir["apos"] = "a'b"

  xml = .json~jsonToXml(dir)
  doc = json~parseXml(xml)
  self~assertEquals("a&b", doc["amp"], "xml roundtrip ampersand")
  self~assertEquals("a<b", doc["lt"], "xml roundtrip less-than")
  self~assertEquals("a>b", doc["gt"], "xml roundtrip greater-than")
  self~assertEquals('a"b', doc["quot"], "xml roundtrip double-quote")
  self~assertEquals("a'b", doc["apos"], "xml roundtrip apostrophe")

  -- Special chars in key names
  dir2 = .directory~new
  dir2["a&b"] = 1
  dir2["c<d"] = 2
  xml2 = .json~jsonToXml(dir2)
  doc2 = json~parseXml(xml2)
  self~assertEquals(1, doc2["a&b"], "xml roundtrip key with ampersand")
  self~assertEquals(2, doc2["c<d"], "xml roundtrip key with less-than")

/*------------------------------------------------------------------------*/
/* XML cross-schema round-trip (XSD -> DTD -> XSD)                        */
/*------------------------------------------------------------------------*/
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
