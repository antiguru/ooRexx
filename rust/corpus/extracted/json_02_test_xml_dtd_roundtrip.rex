/* extracted from json_02::test_xml_dtd_roundtrip */
::routine main public
  json = .json~new
  obj1 = json~fromJSON('{"name":"Alice","age":30,"active":true,"nil_val":null,"arr":[1,2],"nested":{"k":"v"}}')

  xml1 = .json~jsonToXml(obj1, "dtd")
  self~assertTrue(xml1~pos('<!DOCTYPE') > 0, "dtd xml has DOCTYPE")
  self~assertTrue(xml1~pos('xmlns=') == 0, "dtd xml no namespace")

  doc2 = json~parseXml(xml1)
  self~assertTrue(doc2~isA(.directory), "dtd roundtrip type")
  self~assertTrue(json.deepEqual(obj1, doc2), "dtd roundtrip equal")

  self~assertEquals("Alice", doc2["name"], "dtd roundtrip name")
  self~assertEquals(30, doc2["age"], "dtd roundtrip age")
  self~assertTrue(doc2["nil_val"] == .nil, "dtd roundtrip null")

  -- Re-encode stability
  xml2 = .json~jsonToXml(doc2, "dtd")
  doc3 = json~parseXml(xml2)
  self~assertTrue(json.deepEqual(doc2, doc3), "dtd roundtrip stable")

/*------------------------------------------------------------------------*/
/* XML file round-trip                                                    */
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
