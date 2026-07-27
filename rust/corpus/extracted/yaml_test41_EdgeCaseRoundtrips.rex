/* extracted from yaml::test41_EdgeCaseRoundtrips */
::routine main public
  expose parser

  /* 41.1 Special floats: YAML round-trip */
  yaml = "inf: .inf"   || "0A"x || -
         "ninf: -.inf"  || "0A"x || -
         "nan: .nan"    || "0A"x
  doc1 = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc1)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(".inf", doc2["inf"], "special float inf yaml rt")
  self~assertEquals("-.inf", doc2["ninf"], "special float -inf yaml rt")
  self~assertEquals(".nan", doc2["nan"], "special float nan yaml rt")

  /* 41.2 Special floats: XML round-trip (XSD) */
  xml = .Yaml~yamlToXml(doc1, "xsd")
  doc3 = parser~parseXml(xml)
  self~assertEquals(".inf", doc3["inf"], "special float inf xml rt")
  self~assertEquals("-.inf", doc3["ninf"], "special float -inf xml rt")
  self~assertEquals(".nan", doc3["nan"], "special float nan xml rt")

  /* 41.3 Empty string values: YAML round-trip */
  yaml = "empty_sq: ''"   || "0A"x || -
         "empty_dq: " || '"' || '"' || "0A"x || -
         "empty_plain: "   || "0A"x
  doc1 = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc1)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals("", doc2["empty_sq"], "empty sq yaml rt")
  self~assertEquals("", doc2["empty_dq"], "empty dq yaml rt")
  self~assertTrue(doc2["empty_plain"] == .nil, "empty plain is nil")

  /* 41.4 Empty string values: XML round-trip */
  xml = .Yaml~yamlToXml(doc1, "xsd")
  doc3 = parser~parseXml(xml)
  self~assertEquals("", doc3["empty_sq"], "empty sq xml rt")
  self~assertEquals("", doc3["empty_dq"], "empty dq xml rt")

  /* 41.5 Empty collections: YAML round-trip */
  yaml = "empty_seq: []"   || "0A"x || -
         "empty_map: {}"   || "0A"x
  doc1 = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc1)
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(doc2["empty_seq"]~isA(.array), "empty seq yaml rt type")
  self~assertEquals(0, doc2["empty_seq"]~items, "empty seq yaml rt items")
  self~assertTrue(doc2["empty_map"]~isA(.table), "empty map yaml rt type")
  self~assertEquals(0, doc2["empty_map"]~items, "empty map yaml rt items")

  /* 41.6 Empty collections: XML round-trip */
  xml = .Yaml~yamlToXml(doc1, "xsd")
  doc3 = parser~parseXml(xml)
  self~assertTrue(doc3["empty_seq"]~isA(.array), "empty seq xml rt type")
  self~assertEquals(0, doc3["empty_seq"]~items, "empty seq xml rt items")
  self~assertTrue(doc3["empty_map"]~isA(.table), "empty map xml rt type")
  self~assertEquals(0, doc3["empty_map"]~items, "empty map xml rt items")

  /* 41.7 Multiline double-quoted string: YAML round-trip */
  yaml = 'multi: "line one\nline two\nline three"' || "0A"x
  doc1 = parser~parseString(yaml)
  self~assertTrue(doc1["multi"]~pos("0A"x) > 0, "multiline dq has newlines")
  yamlOut = .Yaml~toYaml(doc1)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc1["multi"], doc2["multi"], "multiline dq yaml rt")

  /* 41.8 Multiline double-quoted: XML round-trip */
  xml = .Yaml~yamlToXml(doc1, "xsd")
  doc3 = parser~parseXml(xml)
  self~assertEquals(doc1["multi"], doc3["multi"], "multiline dq xml rt")

  /* 41.9 Directives: round-trip preserves content (directives stripped) */
  yaml = "%YAML 1.2"     || "0A"x || -
         "---"            || "0A"x || -
         "key: value"     || "0A"x
  doc1 = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc1)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals("value", doc2["key"], "directives yaml rt")
  xml = .Yaml~yamlToXml(doc1, "xsd")
  doc3 = parser~parseXml(xml)
  self~assertEquals("value", doc3["key"], "directives xml rt")

  /* 41.10 Single-quoted strings: XML round-trip */
  yaml = "sq: 'It''s a test'"  || "0A"x || -
         "plain_sq: 'hello'"   || "0A"x
  doc1 = parser~parseString(yaml)
  xml = .Yaml~yamlToXml(doc1, "xsd")
  doc3 = parser~parseXml(xml)
  self~assertEquals("It's a test", doc3["sq"], "single-quoted xml rt")
  self~assertEquals("hello", doc3["plain_sq"], "single-quoted plain xml rt")

  /* 41.11 Flow mappings: XML round-trip */
  yaml = "flow: {a: 1, b: 2, c: 3}"  || "0A"x
  doc1 = parser~parseString(yaml)
  xml = .Yaml~yamlToXml(doc1, "xsd")
  doc3 = parser~parseXml(xml)
  self~assertEquals(1, doc3["flow"]["a"], "flow map xml rt a")
  self~assertEquals(2, doc3["flow"]["b"], "flow map xml rt b")
  self~assertEquals(3, doc3["flow"]["c"], "flow map xml rt c")

  /* 41.12 Nested anchors/aliases: XML round-trip via DTD */
  yaml = "defaults: &defs"    || "0A"x || -
         "  color: red"       || "0A"x || -
         "  size: 10"         || "0A"x || -
         "item1:"             || "0A"x || -
         "  <<: *defs"        || "0A"x || -
         "  size: 20"         || "0A"x
  doc1 = parser~parseString(yaml)
  am = parser~anchorMap
  msm = parser~mergeSourceMap
  xml = .Yaml~yamlToXml(doc1, "dtd", am)
  doc3 = parser~parseXml(xml)
  self~assertEquals("red", doc3["item1"]["color"], "anchor merge dtd xml rt color")
  self~assertEquals(20, doc3["item1"]["size"], "anchor merge dtd xml rt size")

  /* 41.13 Multi-document: XML round-trip */
  yaml = "---"            || "0A"x || -
         "first: doc1"    || "0A"x || -
         "---"            || "0A"x || -
         "second: doc2"   || "0A"x
  docs = parser~parseAll(yaml)
  self~assertEquals(2, docs~items, "multi-doc count")
  xml1 = .Yaml~yamlToXml(docs[1], "xsd")
  doc3 = parser~parseXml(xml1)
  self~assertEquals("doc1", doc3["first"], "multi-doc xml rt doc1")
  xml2 = .Yaml~yamlToXml(docs[2], "xsd")
  doc4 = parser~parseXml(xml2)
  self~assertEquals("doc2", doc4["second"], "multi-doc xml rt doc2")

/*------------------------------------------------------------------------*/
/* 42. Tag preservation round-trips                                       */
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
