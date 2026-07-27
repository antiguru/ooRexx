/* extracted from json_02::test_xml_file_roundtrip */
::routine main public
  expose thisLocation
  json = .json~new

  obj1 = json~fromJSON('{"name":"Alice","age":30,"active":true}')

  xmlFile    = thisLocation"test_xml_xsd_tmp.xml"
  xmlDtdFile = thisLocation"test_xml_dtd_tmp.xml"

  .json~jsonToXmlFile(obj1, xmlFile, "xsd")
  doc2 = json~parseXmlFile(xmlFile)
  self~assertTrue(json.deepEqual(obj1, doc2), "xsd file roundtrip equal")

  .json~jsonToXmlFile(obj1, xmlDtdFile, "dtd")
  doc3 = json~parseXmlFile(xmlDtdFile)
  self~assertTrue(json.deepEqual(obj1, doc3), "dtd file roundtrip equal")

  -- Stability: re-write and re-read
  .json~jsonToXmlFile(doc2, xmlFile, "xsd")
  doc4 = json~parseXmlFile(xmlFile)
  self~assertTrue(json.deepEqual(doc2, doc4), "xsd file roundtrip stable")

  .json~jsonToXmlFile(doc3, xmlDtdFile, "dtd")
  doc5 = json~parseXmlFile(xmlDtdFile)
  self~assertTrue(json.deepEqual(doc3, doc5), "dtd file roundtrip stable")

  -- cleanup
  call SysFileDelete xmlFile
  call SysFileDelete xmlDtdFile

/*------------------------------------------------------------------------*/
/* XML type preservation: JsonBoolean                                     */
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
