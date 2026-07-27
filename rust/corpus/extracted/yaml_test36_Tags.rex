/* extracted from yaml::test36_Tags */
::routine main public
  expose parser

  /* !!str tag on a value */
  yaml = "name: !!str Alice"
  doc = parser~parseString(yaml)
  self~assertEquals("Alice", doc["name"], "tag !!str value")

  /* !!int tag on a value */
  yaml = "port: !!int 8080"
  doc = parser~parseString(yaml)
  self~assertEquals(8080, doc["port"], "tag !!int value")

  /* !!null tag */
  yaml = "empty: !!null ~"
  doc = parser~parseString(yaml)
  self~assertTrue(doc["empty"] == .nil, "tag !!null value")

  /* !!bool tag */
  yaml = "flag: !!bool true"
  doc = parser~parseString(yaml)
  self~assertEquals(.YamlBoolean~true, doc["flag"], "tag !!bool value")

  /* Tag on sequence item */
  yaml = "items:" || "0A"x || -
         "  - !!str 42"  || "0A"x || "  - !!int 42"
  doc = parser~parseString(yaml)
  self~assertEquals("42", doc["items"][1], "tag !!str seq item")
  self~assertEquals(42, doc["items"][2], "tag !!int seq item")

  /* Tag on mapping key */
  yaml = "!!str name: Alice"
  doc = parser~parseString(yaml)
  self~assertEquals("Alice", doc["name"], "tag on key")

  /* Tag on block node */
  yaml = "data: !!map" || "0A"x || "  x: 1" || "0A"x || "  y: 2"
  doc = parser~parseString(yaml)
  self~assertEquals(1, doc["data"]["x"], "tag !!map block node x")
  self~assertEquals(2, doc["data"]["y"], "tag !!map block node y")

  /* Tag on flow value */
  yaml = "items: [!!str 42, !!int 7]"
  doc = parser~parseString(yaml)
  self~assertEquals("42", doc["items"][1], "tag !!str flow value")
  self~assertEquals(7, doc["items"][2], "tag !!int flow value")

  /* Custom tag stripped */
  yaml = "data: !custom value123"
  doc = parser~parseString(yaml)
  self~assertEquals("value123", doc["data"], "custom tag stripped")

  /* Verbatim tag stripped */
  yaml = 'data: !<tag:yaml.org,2002:str> hello'
  doc = parser~parseString(yaml)
  self~assertEquals("hello", doc["data"], "verbatim tag stripped")

  /* Tag round-trip */
  yaml = "a: !!str test" || "0A"x || -
         "b: !!int 42"   || "0A"x || -
         "c: !!bool false"
  doc = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals("test", doc2["a"], "tag roundtrip str")
  self~assertEquals(42, doc2["b"], "tag roundtrip int")
  self~assertEquals(.YamlBoolean~false, doc2["c"], "tag roundtrip bool")

/*------------------------------------------------------------------------*/
/* 37. XSLT round-trip (xsltproc / runXSLT.rxj)                          */
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
