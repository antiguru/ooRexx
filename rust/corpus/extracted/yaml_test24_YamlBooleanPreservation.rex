/* extracted from yaml::test24_YamlBooleanPreservation */
::routine main public
  expose parser

  yaml = "btrue: true"     || "0A"x || -
         "bfalse: false"   || "0A"x || -
         "byes: yes"       || "0A"x || -
         "bno: no"         || "0A"x || -
         "bon: on"         || "0A"x || -
         "boff: off"       || "0A"x || -
         "int1: 1"         || "0A"x || -
         "int0: 0"

  doc = parser~parseString(yaml)

  self~assertTrue(doc["btrue"]~isA(.YamlBoolean), "true isA YamlBoolean")
  self~assertTrue(doc["bfalse"]~isA(.YamlBoolean), "false isA YamlBoolean")
  self~assertTrue(doc["byes"]~isA(.YamlBoolean), "yes isA YamlBoolean")
  self~assertTrue(doc["bno"]~isA(.YamlBoolean), "no isA YamlBoolean")
  self~assertTrue(doc["bon"]~isA(.YamlBoolean), "on isA YamlBoolean")
  self~assertTrue(doc["boff"]~isA(.YamlBoolean), "off isA YamlBoolean")

  self~assertFalse(doc["int1"]~isA(.YamlBoolean), "1 not YamlBoolean")
  self~assertFalse(doc["int0"]~isA(.YamlBoolean), "0 not YamlBoolean")

  self~assertEquals(1, doc["btrue"], "true == 1")
  self~assertEquals(0, doc["bfalse"], "false == 0")
  self~assertEquals(1, doc["byes"], "yes == 1")
  self~assertEquals(0, doc["bno"], "no == 0")

  self~assertEquals("true", doc["btrue"]~makeYAML, "true makeYAML")
  self~assertEquals("false", doc["bfalse"]~makeYAML, "false makeYAML")

  self~assertTrue(.Yaml~true~isA(.YamlBoolean), ".Yaml~true")
  self~assertTrue(.Yaml~false~isA(.YamlBoolean), ".Yaml~false")
  self~assertEquals(1, .Yaml~true, ".Yaml~true value")
  self~assertEquals(0, .Yaml~false, ".Yaml~false value")

  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(doc2["btrue"]~isA(.YamlBoolean), "yaml roundtrip btrue")
  self~assertTrue(doc2["bfalse"]~isA(.YamlBoolean), "yaml roundtrip bfalse")
  self~assertFalse(doc2["int1"]~isA(.YamlBoolean), "yaml roundtrip int1")
  self~assertTrue(YAML.deepEqual(doc, doc2), "yaml roundtrip equal")

  xml = .Yaml~yamlToXml(doc, "xsd")
  self~assertTrue(xml~pos('type="bool"') > 0, "xml has bool type")
  doc3 = parser~parseXml(xml)
  self~assertTrue(doc3["btrue"]~isA(.YamlBoolean), "xml roundtrip btrue isA")
  self~assertTrue(doc3["bfalse"]~isA(.YamlBoolean), "xml roundtrip bfalse isA")
  self~assertFalse(doc3["int1"]~isA(.YamlBoolean), "xml roundtrip int1 not")
  self~assertTrue(YAML.deepEqual(doc, doc3), "xml roundtrip equal")

/*------------------------------------------------------------------------*/
/* 25. Anchor/alias round-trip                                            */
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
