/* extracted from json_02::test_xml_string_preservation */
::routine main public
  json = .json~new
  dir = .directory~new
  dir["jstr"] = .JsonString~new("42")
  dir["num"] = 42
  dir["plain"] = "hello"

  xml = .json~jsonToXml(dir)
  doc = json~parseXml(xml)

  self~assertTrue(doc["jstr"]~isA(.JsonString), "xml roundtrip jstr isA")
  self~assertEquals("42", doc["jstr"], "xml roundtrip jstr value")
  self~assertFalse(doc["num"]~isA(.JsonString), "xml roundtrip num not string")
  self~assertEquals(42, doc["num"], "xml roundtrip num value")
  self~assertTrue(doc["plain"]~isA(.JsonString), "xml roundtrip plain isA")

/*------------------------------------------------------------------------*/
/* XML empty collections and null                                         */
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
