/* extracted from json_02::test_xml_xsd_roundtrip */
::routine main public
  json = .json~new
  obj1 = json~fromJSON('{"name":"Alice","age":30,"active":true,"nil_val":null,"arr":[1,2],"nested":{"k":"v"}}')

  xml1 = .json~jsonToXml(obj1, "xsd")
  self~assertTrue(xml1~pos('<?xml') > 0, "xsd xml has declaration")
  self~assertTrue(xml1~pos('xmlns=') > 0, "xsd xml has namespace")
  self~assertTrue(xml1~pos('<json') > 0, "xsd xml has json element")

  doc2 = json~parseXml(xml1)
  self~assertTrue(doc2~isA(.directory), "xsd roundtrip type")
  self~assertTrue(json.deepEqual(obj1, doc2), "xsd roundtrip equal")

  self~assertEquals("Alice", doc2["name"], "xsd roundtrip name")
  self~assertEquals(30, doc2["age"], "xsd roundtrip age")
  self~assertTrue(doc2["active"]~isA(.JsonBoolean), "xsd roundtrip bool isA")
  self~assertTrue(doc2["active"] = .true, "xsd roundtrip bool value")
  self~assertTrue(doc2["nil_val"] == .nil, "xsd roundtrip null")
  self~assertEquals(2, doc2["arr"]~items, "xsd roundtrip arr items")
  self~assertEquals("v", doc2["nested"]["k"], "xsd roundtrip nested")

  -- Re-encode stability
  xml2 = .json~jsonToXml(doc2, "xsd")
  doc3 = json~parseXml(xml2)
  self~assertTrue(json.deepEqual(doc2, doc3), "xsd roundtrip stable")

/*------------------------------------------------------------------------*/
/* XML round-trip via DTD                                                 */
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
