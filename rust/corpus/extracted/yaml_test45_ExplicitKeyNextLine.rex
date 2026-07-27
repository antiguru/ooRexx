/* extracted from yaml::test45_ExplicitKeyNextLine */
::routine main public

  /* 45.1 ? with scalar key on next line */
  yaml1 = "?" || "0A"x || "  my_key" || "0A"x || ": value1"
  doc1 = .Yaml~new~parseString(yaml1)
  self~assertEquals("value1", doc1["my_key"], "? scalar key next line")

  /* 45.2 ? with flow sequence key on next line */
  yaml2 = "?" || "0A"x || "  [a, b]" || "0A"x || ": value2"
  doc2 = .Yaml~new~parseString(yaml2)
  found = .false
  sup = doc2~supplier
  do while sup~available
    if sup~index~isA(.array) then do
      self~assertEquals(2, sup~index~items, "? seq key items")
      self~assertEquals("a", sup~index[1], "? seq key [1]")
      self~assertEquals("b", sup~index[2], "? seq key [2]")
      self~assertEquals("value2", sup~item, "? seq key value")
      found = .true
    end
    sup~next
  end
  self~assertTrue(found, "? seq key next line found")

  /* 45.3 ? with block mapping key on next lines */
  yaml3 = "?" || "0A"x || "  x: 1" || "0A"x || "  y: 2" || "0A"x || ": complex_value"
  doc3 = .Yaml~new~parseString(yaml3)
  found3 = .false
  sup3 = doc3~supplier
  do while sup3~available
    if sup3~index~isA(.table) then do
      self~assertEquals(1, sup3~index["x"], "? map key x")
      self~assertEquals(2, sup3~index["y"], "? map key y")
      self~assertEquals("complex_value", sup3~item, "? map key value")
      found3 = .true
    end
    sup3~next
  end
  self~assertTrue(found3, "? map key next line found")

  /* 45.4 ? alone = null key */
  yaml4 = "?" || "0A"x || ": null_key_value"
  doc4 = .Yaml~new~parseString(yaml4)
  self~assertEquals("null_key_value", doc4~at(.nil), "? null key")

  /* 45.5 YAML round-trip */
  yaml_rt = .Yaml~toYaml(doc1)
  doc1_rt = .Yaml~new~parseString(yaml_rt)
  self~assertEquals("value1", doc1_rt["my_key"], "? scalar key YAML rt")

  /* 45.6 XML round-trip */
  xml = .Yaml~yamlToXml(doc1)
  doc1_x = .Yaml~new~parseXml(xml)
  self~assertEquals("value1", doc1_x["my_key"], "? scalar key XML rt")

/*========================================================================*/
/* Group 46 — Multi-line quoted strings (P4)                             */
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
