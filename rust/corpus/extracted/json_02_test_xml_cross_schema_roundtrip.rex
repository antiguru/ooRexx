/* extracted from json_02::test_xml_cross_schema_roundtrip */
::routine main public
  json = .json~new
  obj = json~fromJSON('{"a":1,"b":"hello","c":true,"d":null,"e":[1,2,3]}')

  xmlXsd = .json~jsonToXml(obj, "xsd")
  docXsd = json~parseXml(xmlXsd)
  xmlDtd = .json~jsonToXml(docXsd, "dtd")
  docDtd = json~parseXml(xmlDtd)
  self~assertTrue(json.deepEqual(docXsd, docDtd), "xsd->dtd cross roundtrip")

  -- And back to XSD
  xmlXsd2 = .json~jsonToXml(docDtd, "xsd")
  docXsd2 = json~parseXml(xmlXsd2)
  self~assertTrue(json.deepEqual(docDtd, docXsd2), "dtd->xsd cross roundtrip")


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
