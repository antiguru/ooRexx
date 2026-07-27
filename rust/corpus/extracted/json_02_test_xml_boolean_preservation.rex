/* extracted from json_02::test_xml_boolean_preservation */
::routine main public
  json = .json~new
  dir = .directory~new
  dir["bt"] = .JsonBoolean~true
  dir["bf"] = .JsonBoolean~false
  dir["one"] = 1
  dir["zero"] = 0

  xml = .json~jsonToXml(dir, "xsd")
  self~assertTrue(xml~pos('<boolean>true</boolean>') > 0, "xml has bool true")
  self~assertTrue(xml~pos('<boolean>false</boolean>') > 0, "xml has bool false")

  doc = json~parseXml(xml)
  self~assertTrue(doc["bt"]~isA(.JsonBoolean), "xml roundtrip bt isA")
  self~assertTrue(doc["bf"]~isA(.JsonBoolean), "xml roundtrip bf isA")
  self~assertTrue(doc["bt"] = .true, "xml roundtrip bt value")
  self~assertTrue(doc["bf"] = .false, "xml roundtrip bf value")
  self~assertFalse(doc["one"]~isA(.JsonBoolean), "xml roundtrip one not bool")
  self~assertFalse(doc["zero"]~isA(.JsonBoolean), "xml roundtrip zero not bool")

/*------------------------------------------------------------------------*/
/* XML type preservation: JsonString                                      */
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
