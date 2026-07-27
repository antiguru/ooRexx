/* extracted from yaml::test30_FlowStyleEmission */
::routine main public
  expose parser

  /* 30a. Simple sequence emitted as flow when nested */
  yaml = "colors:"         || "0A"x || -
         "  - red"         || "0A"x || -
         "  - green"       || "0A"x || -
         "  - blue"
  doc = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos("[") > 0, "flow seq emitted")
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "flow seq roundtrip")

  /* 30b. 4 items (max for flow) */
  yaml = "nums:"       || "0A"x || -
         "  - 1"       || "0A"x || -
         "  - 2"       || "0A"x || -
         "  - 3"       || "0A"x || -
         "  - 4"
  doc = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos("[") > 0, "flow seq 4 items")
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "flow seq 4 roundtrip")

  /* 30c. 5 items stays block */
  yaml = "nums:"       || "0A"x || -
         "  - 1"       || "0A"x || -
         "  - 2"       || "0A"x || -
         "  - 3"       || "0A"x || -
         "  - 4"       || "0A"x || -
         "  - 5"
  doc = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos("- 1") > 0, "block seq 5 items")
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "block seq 5 roundtrip")

  /* 30d. Nested collection stays block */
  yaml = "data:"              || "0A"x || -
         "  - name: Alice"    || "0A"x || -
         "  - name: Bob"
  doc = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos("[") = 0, "block seq nested maps")
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "block seq nested roundtrip")

  /* 30e. Sequence with null */
  yaml = "vals:"     || "0A"x || -
         "  - hello" || "0A"x || -
         "  - null"  || "0A"x || -
         "  - world"
  doc = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "seq with null roundtrip")

  /* 30f. Empty collections */
  yaml = "em: {}" || "0A"x || "el: []"
  doc = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos("{}") > 0, "empty map stays {}")
  self~assertTrue(yamlOut~pos("[]") > 0, "empty list stays []")

  /* 30g. Flow sequence XML round-trip */
  yaml = "colors:"  || "0A"x || -
         "  - red"  || "0A"x || -
         "  - blue"
  doc = parser~parseString(yaml)
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc3 = parser~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc3), "flow seq xml xsd rt")
  xml2 = .Yaml~yamlToXml(doc, "dtd")
  doc4 = parser~parseXml(xml2)
  self~assertTrue(YAML.deepEqual(doc, doc4), "flow seq xml dtd rt")

  /* 30h. Top-level sequence stays block */
  yaml = "- alpha" || "0A"x || "- beta"
  doc = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos("[") = 0, "top seq stays block")
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "top seq roundtrip")

/*------------------------------------------------------------------------*/
/* 31. Anchor ordering in toYaml                                          */
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
