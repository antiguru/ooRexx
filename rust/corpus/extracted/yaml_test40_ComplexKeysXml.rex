/* extracted from yaml::test40_ComplexKeysXml */
::routine main public
  expose parser

  /* 40.1 XSD round-trip: sequence as key */
  yaml = "? [a, b]" || "0A"x || ": value1" || "0A"x
  doc = parser~parseString(yaml)
  xml = .Yaml~yamlToXml(doc, "xsd")
  self~assertTrue(xml~pos("<sequence>") > 0, -
    "xml seq key: has sequence element")
  doc2 = parser~parseXml(xml)
  self~assertEquals(1, doc2~items, "xml seq key: one entry")
  sup = doc2~supplier
  Do While sup~available
    self~assertTrue(sup~index~isA(.array), "xml seq key: key is array")
    self~assertEquals("a", sup~index[1], "xml seq key: key[1]")
    self~assertEquals("b", sup~index[2], "xml seq key: key[2]")
    self~assertEquals("value1", sup~item, "xml seq key: value")
    sup~next
  End

  /* 40.2 DTD round-trip: sequence as key */
  xml = .Yaml~yamlToXml(doc, "dtd")
  doc3 = parser~parseXml(xml)
  sup = doc3~supplier
  Do While sup~available
    self~assertTrue(sup~index~isA(.array), "dtd seq key: key is array")
    self~assertEquals("a", sup~index[1], "dtd seq key: key[1]")
    self~assertEquals("value1", sup~item, "dtd seq key: value")
    sup~next
  End

  /* 40.3 XSD round-trip: mapping as key */
  yaml = "? {x: 1}" || "0A"x || ": value2" || "0A"x
  doc = parser~parseString(yaml)
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc2 = parser~parseXml(xml)
  self~assertEquals(1, doc2~items, "xml map key: one entry")
  sup = doc2~supplier
  Do While sup~available
    self~assertTrue(sup~index~isA(.table), "xml map key: key is table")
    self~assertEquals(1, sup~index["x"], "xml map key: key[x]")
    self~assertEquals("value2", sup~item, "xml map key: value")
    sup~next
  End

  /* 40.4 XSD round-trip: complex key with null value */
  yaml = "? [x, y]" || "0A"x
  doc = parser~parseString(yaml)
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc2 = parser~parseXml(xml)
  sup = doc2~supplier
  Do While sup~available
    self~assertTrue(sup~index~isA(.array), -
      "xml null val: key is array")
    self~assertTrue(sup~item == .nil, -
      "xml null val: value is nil")
    sup~next
  End

  /* 40.5 XSD round-trip: mixed simple and complex keys */
  yaml = "simple: val1" || "0A"x || -
         "? [a, b]"      || "0A"x || -
         ": val2"         || "0A"x
  doc = parser~parseString(yaml)
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc2 = parser~parseXml(xml)
  self~assertEquals("val1", doc2["simple"], "xml mixed keys: simple")
  found = .false
  sup = doc2~supplier
  Do While sup~available
    If sup~index~isA(.array) Then Do
      self~assertEquals("a", sup~index[1], "xml mixed keys: complex[1]")
      self~assertEquals("val2", sup~item, "xml mixed keys: complex val")
      found = .true
    End
    sup~next
  End
  self~assertTrue(found, "xml mixed keys: found complex")

  /* 40.6 XML stability: complex key XML → parse → XML → parse */
  yaml = "? [1, 2]" || "0A"x || ": result" || "0A"x
  doc = parser~parseString(yaml)
  xml1 = .Yaml~yamlToXml(doc, "xsd")
  doc2 = parser~parseXml(xml1)
  xml2 = .Yaml~yamlToXml(doc2, "xsd")
  doc3 = parser~parseXml(xml2)
  self~assertTrue(YAML.deepEqual(doc2, doc3), "xml complex key stable")

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
