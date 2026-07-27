/* extracted from yaml::test21_XmlXsdRoundtrip */
::routine main public
  expose parser thisLocation

  inFile  = thisLocation"test_all_constructs.yaml"
  outFile = thisLocation"test21_xsd_out.yaml"

  doc1 = parser~parseFile(inFile)

  xml1 = .Yaml~yamlToXml(doc1, "xsd")
  self~assertTrue(xml1~pos('<?xml') > 0, "xsd xml has declaration")
  self~assertTrue(xml1~pos('xmlns=') > 0, "xsd xml has namespace")
  self~assertTrue(xml1~pos('<yaml') > 0, "xsd xml has yaml element")

  doc2 = parser~parseXml(xml1)
  self~assertTrue(doc2~isA(.table), "xsd roundtrip type")
  self~assertTrue(YAML.deepEqual(doc1, doc2), "xsd roundtrip equal")

  self~assertEquals("hello world", doc2["plain"], "xsd roundtrip plain")
  self~assertEquals(42, doc2["integer"], "xsd roundtrip integer")
  self~assertEquals(3.14, doc2["float"], "xsd roundtrip float")
  self~assertEquals(.nil, doc2["null_value"], "xsd roundtrip null")
  self~assertEquals("", doc2["empty_string"], "xsd roundtrip empty s")
  self~assertEquals("deep", doc2["nested_map"]["level1"]["level2"]["level3"], "xsd roundtrip nested")
  self~assertEquals(3, doc2["simple_list"]~items, "xsd roundtrip list")
  self~assertEquals("beta", doc2["simple_list"][2], "xsd roundtrip list[2]")
  self~assertEquals("Alice", doc2["list_of_maps"][1]["name"], "xsd roundtrip lom")
  self~assertEquals(0, doc2["empty_map"]~items, "xsd roundtrip empty m")
  self~assertEquals(0, doc2["empty_list"]~items, "xsd roundtrip empty l")
  self~assertTrue(doc2["multiline"]~pos("multi-line") > 0, "xsd roundtrip multi")
  self~assertTrue(doc2["special_chars"]~pos("09"x) > 0, "xsd roundtrip special")
  self~assertEquals("admin", doc2["mixed_nesting"]["users"][1]["name"], "xsd roundtrip users")
  self~assertEquals("write", doc2["mixed_nesting"]["users"][1]["roles"][2], "xsd roundtrip roles")

  xml2 = .Yaml~yamlToXml(doc2, "xsd")
  doc3 = parser~parseXml(xml2)
  self~assertTrue(YAML.deepEqual(doc2, doc3), "xsd roundtrip stable")

  .Yaml~toYamlFile(doc2, outFile)
  doc4 = parser~parseFile(outFile)
  self~assertTrue(YAML.deepEqual(doc1, doc4), "xsd yaml output equal")

  -- cleanup
  call SysFileDelete outFile

/*------------------------------------------------------------------------*/
/* 22. XML round-trip via DTD                                             */
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
