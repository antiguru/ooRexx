/* extracted from yaml::test42_TagPreservationRoundtrips */
::routine main public
  expose parser

  /* Tag-preserving parser */
  tp = .Yaml~new(, .true)

  /* 42.1 !!str scalar: parse produces YamlTagged */
  doc = tp~parseString('!!str 42')
  self~assertTrue(doc~isA(.YamlTagged), "!!str produces YamlTagged")
  self~assertEquals("!!str", doc~tag, "!!str tag preserved")
  self~assertEquals("42", doc~value, "!!str value is string 42")

  /* 42.2 !!int scalar */
  doc = tp~parseString('!!int 42')
  self~assertEquals("!!int", doc~tag, "!!int tag preserved")
  self~assertEquals(42, doc~value, "!!int value is integer")

  /* 42.3 !custom scalar */
  doc = tp~parseString('!custom hello')
  self~assertEquals("!custom", doc~tag, "!custom tag preserved")
  self~assertEquals("hello", doc~value, "!custom value")

  /* 42.4 Verbatim tag */
  doc = tp~parseString('!<tag:yaml.org,2002:str> hello')
  self~assertEquals("!<tag:yaml.org,2002:str>", doc~tag, "verbatim tag preserved")
  self~assertEquals("hello", doc~value, "verbatim tag value")

  /* 42.5 !!map on block mapping */
  yaml = "!!map" || "0A"x || "a: 1" || "0A"x
  doc = tp~parseString(yaml)
  self~assertTrue(doc~isA(.YamlTagged), "!!map produces YamlTagged")
  self~assertEquals("!!map", doc~tag, "!!map tag")
  self~assertTrue(doc~value~isA(.table), "!!map value is table")
  self~assertEquals(1, doc~value["a"], "!!map inner value")

  /* 42.6 !!seq on block sequence */
  yaml = "!!seq" || "0A"x || "- one" || "0A"x || "- two" || "0A"x
  doc = tp~parseString(yaml)
  self~assertEquals("!!seq", doc~tag, "!!seq tag")
  self~assertTrue(doc~value~isA(.array), "!!seq value is array")
  self~assertEquals("one", doc~value[1], "!!seq item 1")

  /* 42.7 Tags in flow collections */
  yaml = "[!!str 42, !!int 7]"
  doc = tp~parseString(yaml)
  self~assertEquals("!!str", doc[1]~tag, "flow seq !!str tag")
  self~assertEquals("!!int", doc[2]~tag, "flow seq !!int tag")

  /* 42.8 Tag on mapping key */
  yaml = "plain: x" || "0A"x || "!!str tagged: y" || "0A"x
  doc = tp~parseString(yaml)
  found = .false
  sup = doc~supplier
  do while sup~available
    k = sup~index
    if k~isA(.YamlTagged) then do
      self~assertEquals("!!str", k~tag, "tag on key")
      self~assertEquals("tagged", k~value, "tagged key value")
      found = .true
    end
    sup~next
  end
  self~assertTrue(found, "tagged key found")

  /* 42.9 YAML round-trip with tags */
  doc = tp~parseString('!!str 42')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = tp~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "YAML rt !!str")

  /* 42.10 YAML round-trip: !!map */
  yaml = "!!map" || "0A"x || "x: 1" || "0A"x
  doc = tp~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  doc2 = tp~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "YAML rt !!map")

  /* 42.11 XML round-trip XSD: !!str scalar */
  doc = tp~parseString('!!str 42')
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc2 = tp~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc2), "XML XSD rt !!str")

  /* 42.12 XML round-trip DTD: !!str scalar */
  doc = tp~parseString('!!str 42')
  xml = .Yaml~yamlToXml(doc, "dtd")
  doc2 = tp~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc2), "XML DTD rt !!str")

  /* 42.13 XML round-trip XSD: !custom tag */
  doc = tp~parseString('!custom hello')
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc2 = tp~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc2), "XML XSD rt !custom")

  /* 42.14 XML round-trip: verbatim tag */
  doc = tp~parseString('!<tag:yaml.org,2002:str> hello')
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc2 = tp~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc2), "XML XSD rt verbatim tag")

  /* 42.15 XML round-trip: !!map on mapping */
  yaml = "!!map" || "0A"x || "a: 1" || "0A"x
  doc = tp~parseString(yaml)
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc2 = tp~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc2), "XML XSD rt !!map")

  /* 42.16 XML round-trip DTD: !!map */
  yaml = "!!map" || "0A"x || "a: 1" || "0A"x
  doc = tp~parseString(yaml)
  xml = .Yaml~yamlToXml(doc, "dtd")
  doc2 = tp~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc2), "XML DTD rt !!map")

  /* 42.17 XML round-trip: !!seq on sequence */
  yaml = "!!seq" || "0A"x || "- one" || "0A"x || "- two" || "0A"x
  doc = tp~parseString(yaml)
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc2 = tp~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc2), "XML XSD rt !!seq")

  /* 42.18 Full chain round-trip: YAML -> parse -> XML -> parse -> YAML */
  doc = tp~parseString('!!str 42')
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc2 = tp~parseXml(xml)
  yamlOut = .Yaml~toYaml(doc2)
  doc3 = tp~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc3), "full chain rt !!str")

  /* 42.19 preserveTags = .false (default) — tags stripped as before */
  pDef = .Yaml~new
  doc = pDef~parseString('!!str 42')
  self~assertFalse(doc~isA(.YamlTagged), "default no YamlTagged")
  self~assertEquals("42", doc, "default strips tag")

  /* 42.20 Tags on sequence items in XML round-trip */
  yaml = "- !!str foo" || "0A"x || "- !!int 7" || "0A"x
  doc = tp~parseString(yaml)
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc2 = tp~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc2), "XML XSD rt tagged seq items")

  /* 42.21 Tag + anchor combination */
  yaml = "&myA !!str hello" || "0A"x
  doc = tp~parseString(yaml)
  am = tp~anchorMap
  self~assertTrue(doc~isA(.YamlTagged), "tag+anchor YamlTagged")
  self~assertEquals("!!str", doc~tag, "tag+anchor tag")
  self~assertEquals("hello", doc~value, "tag+anchor value")
  xml = .Yaml~yamlToXml(doc, "xsd", am)
  doc2 = tp~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc2), "XML rt tag+anchor")

  /* 42.22 YAML.deepEqual: both tagged same */
  a = .YamlTagged~new("!!str", "42")
  b = .YamlTagged~new("!!str", "42")
  self~assertTrue(YAML.deepEqual(a, b), "deepEqual same tagged")

  /* 42.23 YAML.deepEqual: different tags */
  a = .YamlTagged~new("!!str", "42")
  b = .YamlTagged~new("!!int", 42)
  self~assertFalse(YAML.deepEqual(a, b), "deepEqual diff tags")

  /* 42.24 YAML.deepEqual: tagged vs untagged */
  a = .YamlTagged~new("!!str", "42")
  self~assertFalse(YAML.deepEqual(a, "42"), "deepEqual tagged vs plain")

  /* 42.25 YAML.deepEqual: untagged vs tagged */
  self~assertFalse(YAML.deepEqual("42", a), "deepEqual plain vs tagged")

/*========================================================================*/
/* Group 43 — Unicode escape shortcuts \N, \_, \L, \P (P8)               */
/*========================================================================*/
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
