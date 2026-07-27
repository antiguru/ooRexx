/* extracted from yaml::test23_XmlFileRoundtrip */
::routine main public
  expose parser thisLocation

  inFile     = thisLocation"test_all_constructs.yaml"
  xmlFile    = thisLocation"test23_xsd.xml"
  xmlDtdFile = thisLocation"test23_dtd.xml"

  doc1 = parser~parseFile(inFile)

  .Yaml~yamlToXmlFile(doc1, xmlFile, "xsd")
  doc8 = parser~parseXmlFile(xmlFile)
  self~assertTrue(YAML.deepEqual(doc1, doc8), "xsd file roundtrip equal")

  .Yaml~yamlToXmlFile(doc1, xmlDtdFile, "dtd")
  doc9 = parser~parseXmlFile(xmlDtdFile)
  self~assertTrue(YAML.deepEqual(doc1, doc9), "dtd file roundtrip equal")

  .Yaml~yamlToXmlFile(doc8, xmlFile, "xsd")
  doc10 = parser~parseXmlFile(xmlFile)
  self~assertTrue(YAML.deepEqual(doc8, doc10), "xsd file roundtrip stable")

  .Yaml~yamlToXmlFile(doc9, xmlDtdFile, "dtd")
  doc11 = parser~parseXmlFile(xmlDtdFile)
  self~assertTrue(YAML.deepEqual(doc9, doc11), "dtd file roundtrip stable")

  -- cleanup
  call SysFileDelete xmlFile
  call SysFileDelete xmlDtdFile

/*------------------------------------------------------------------------*/
/* 24. YamlBoolean type preservation                                      */
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
