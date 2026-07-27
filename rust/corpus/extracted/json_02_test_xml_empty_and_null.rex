/* extracted from json_02::test_xml_empty_and_null */
::routine main public
  json = .json~new
  dir = .directory~new
  dir["eo"] = .directory~new
  dir["ea"] = .array~new
  dir["nv"] = .nil

  xml = .json~jsonToXml(dir)
  self~assertTrue(xml~pos('<object/>') > 0, "xml has empty object")
  self~assertTrue(xml~pos('<array/>') > 0, "xml has empty array")
  self~assertTrue(xml~pos('<null/>') > 0, "xml has null element")

  doc = json~parseXml(xml)
  self~assertEquals(0, doc["eo"]~items, "xml roundtrip empty object")
  self~assertEquals(0, doc["ea"]~items, "xml roundtrip empty array")
  self~assertTrue(doc["nv"] == .nil, "xml roundtrip null")

/*------------------------------------------------------------------------*/
/* XML array at root                                                      */
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
