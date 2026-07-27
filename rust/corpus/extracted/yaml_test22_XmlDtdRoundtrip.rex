/* extracted from yaml::test22_XmlDtdRoundtrip */
::routine main public
  expose parser thisLocation

  inFile     = thisLocation"test_all_constructs.yaml"
  outDtdFile = thisLocation"test22_dtd_out.yaml"

  doc1 = parser~parseFile(inFile)

  xml3 = .Yaml~yamlToXml(doc1, "dtd")
  self~assertTrue(xml3~pos('<!DOCTYPE') > 0, "dtd xml has DOCTYPE")
  self~assertTrue(xml3~pos('xmlns=') == 0, "dtd xml no namespace")

  doc5 = parser~parseXml(xml3)
  self~assertTrue(doc5~isA(.table), "dtd roundtrip type")
  self~assertTrue(YAML.deepEqual(doc1, doc5), "dtd roundtrip equal")

  self~assertEquals("hello world", doc5["plain"], "dtd roundtrip plain")
  self~assertEquals(42, doc5["integer"], "dtd roundtrip integer")
  self~assertEquals("deep", doc5["nested_map"]["level1"]["level2"]["level3"], "dtd roundtrip nested")
  self~assertEquals("beta", doc5["simple_list"][2], "dtd roundtrip list")
  self~assertEquals("Alice", doc5["list_of_maps"][1]["name"], "dtd roundtrip lom")
  self~assertEquals(0, doc5["empty_map"]~items, "dtd roundtrip empty m")
  self~assertEquals(0, doc5["empty_list"]~items, "dtd roundtrip empty l")
  self~assertEquals("admin", doc5["mixed_nesting"]["users"][1]["name"], "dtd roundtrip users")

  xml4 = .Yaml~yamlToXml(doc5, "dtd")
  doc6 = parser~parseXml(xml4)
  self~assertTrue(YAML.deepEqual(doc5, doc6), "dtd roundtrip stable")

  .Yaml~toYamlFile(doc5, outDtdFile)
  doc7 = parser~parseFile(outDtdFile)
  self~assertTrue(YAML.deepEqual(doc1, doc7), "dtd yaml output equal")

  -- cleanup
  call SysFileDelete outDtdFile

/*------------------------------------------------------------------------*/
/* 23. XML file round-trip (yamlToXmlFile / parseXmlFile)                 */
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
