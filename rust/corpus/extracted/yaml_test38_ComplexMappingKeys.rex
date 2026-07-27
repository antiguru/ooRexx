/* extracted from yaml::test38_ComplexMappingKeys */
::routine main public
  expose parser

  /* 38.1 Flow sequence as key */
  yaml = "? [a, b]" || "0A"x || ": value1" || "0A"x
  doc = parser~parseString(yaml)
  self~assertEquals(1, doc~items, "flow seq key: one entry")
  sup = doc~supplier
  Do While sup~available
    self~assertTrue(sup~index~isA(.array), "flow seq key: key is array")
    self~assertEquals(2, sup~index~items, "flow seq key: key has 2 items")
    self~assertEquals("a", sup~index[1], "flow seq key: key[1]")
    self~assertEquals("b", sup~index[2], "flow seq key: key[2]")
    self~assertEquals("value1", sup~item, "flow seq key: value")
    sup~next
  End

  /* 38.2 Flow mapping as key */
  yaml = "? {x: 1}" || "0A"x || ": value2" || "0A"x
  doc = parser~parseString(yaml)
  self~assertEquals(1, doc~items, "flow map key: one entry")
  sup = doc~supplier
  Do While sup~available
    self~assertTrue(sup~index~isA(.table), "flow map key: key is table")
    self~assertEquals(1, sup~index["x"], "flow map key: key[x]")
    self~assertEquals("value2", sup~item, "flow map key: value")
    sup~next
  End

  /* 38.3 Block sequence as key */
  yaml = "?" || "0A"x || "  - item1" || "0A"x || -
         "  - item2" || "0A"x || ": value3" || "0A"x
  doc = parser~parseString(yaml)
  self~assertEquals(1, doc~items, "block seq key: one entry")
  sup = doc~supplier
  Do While sup~available
    self~assertTrue(sup~index~isA(.array), "block seq key: key is array")
    self~assertEquals("item1", sup~index[1], "block seq key: key[1]")
    self~assertEquals("item2", sup~index[2], "block seq key: key[2]")
    self~assertEquals("value3", sup~item, "block seq key: value")
    sup~next
  End

  /* 38.4 Scalar complex key */
  yaml = "? scalar_key" || "0A"x || ": value4" || "0A"x
  doc = parser~parseString(yaml)
  self~assertEquals("value4", doc["scalar_key"], "scalar complex key")

  /* 38.5 Complex key with null value */
  yaml = "? [a, b]" || "0A"x
  doc = parser~parseString(yaml)
  self~assertEquals(1, doc~items, "complex key null value: one entry")
  sup = doc~supplier
  Do While sup~available
    self~assertTrue(sup~index~isA(.array), -
      "complex key null value: key is array")
    self~assertTrue(sup~item == .nil, -
      "complex key null value: value is nil")
    sup~next
  End

  /* 38.6 Mixed simple and complex keys */
  yaml = "simple: val1" || "0A"x || -
         "? [x, y]" || "0A"x || ": val2" || "0A"x
  doc = parser~parseString(yaml)
  self~assertEquals(2, doc~items, "mixed keys: two entries")
  self~assertEquals("val1", doc["simple"], "mixed keys: simple key")

  /* 38.7 Multiple complex keys */
  yaml = "? [a, b]" || "0A"x || ": val1" || "0A"x || -
         "? [c, d]" || "0A"x || ": val2" || "0A"x
  doc = parser~parseString(yaml)
  self~assertEquals(2, doc~items, "multiple complex keys: two entries")

  /* 38.8 Round-trip: flow sequence key */
  yaml = "? [a, b]" || "0A"x || ": value1" || "0A"x
  doc = parser~parseString(yaml)
  out = .Yaml~toYaml(doc)
  doc2 = parser~parseString(out)
  self~assertTrue(YAML.deepEqual(doc, doc2), "round-trip flow seq key")

  /* 38.9 Round-trip: flow mapping key */
  yaml = "? {x: 1}" || "0A"x || ": value2" || "0A"x
  doc = parser~parseString(yaml)
  out = .Yaml~toYaml(doc)
  doc2 = parser~parseString(out)
  self~assertTrue(YAML.deepEqual(doc, doc2), "round-trip flow map key")

  /* 38.10 Round-trip: block sequence key */
  yaml = "?" || "0A"x || "  - item1" || "0A"x || -
         "  - item2" || "0A"x || ": value3" || "0A"x
  doc = parser~parseString(yaml)
  out = .Yaml~toYaml(doc)
  doc2 = parser~parseString(out)
  self~assertTrue(YAML.deepEqual(doc, doc2), "round-trip block seq key")

  /* 38.11 Round-trip: mixed simple + complex keys */
  yaml = "simple: val1" || "0A"x || -
         "? [x, y]" || "0A"x || ": val2" || "0A"x
  doc = parser~parseString(yaml)
  out = .Yaml~toYaml(doc)
  doc2 = parser~parseString(out)
  self~assertTrue(YAML.deepEqual(doc, doc2), "round-trip mixed keys")

  /* 38.12 Round-trip: programmatic creation with array key */
  doc = .table~new
  doc[.array~of("a", "b")] = "value1"
  out = .Yaml~toYaml(doc)
  doc2 = parser~parseString(out)
  self~assertTrue(YAML.deepEqual(doc, doc2), -
    "round-trip programmatic array key")

  /* 38.13 Round-trip: programmatic creation with table key */
  doc = .table~new
  tKey = .table~new; tKey["x"] = 1
  doc[tKey] = "value2"
  out = .Yaml~toYaml(doc)
  doc2 = parser~parseString(out)
  self~assertTrue(YAML.deepEqual(doc, doc2), -
    "round-trip programmatic table key")

  /* 38.14 Emitter uses ? syntax for non-string keys */
  doc = .table~new
  doc[.array~of("a", "b")] = "value1"
  out = .Yaml~toYaml(doc)
  self~assertTrue(out~pos("? ") > 0, "emitter uses ? for array key")
  self~assertTrue(out~pos(": value1") > 0, "emitter emits value after :")

  /* 38.15 YAML.deepEqual with non-string keys: equal */
  a = .table~new; a[.array~of(1, 2)] = "v1"
  b = .table~new; b[.array~of(1, 2)] = "v1"
  self~assertTrue(YAML.deepEqual(a, b), "deepEqual non-string keys equal")

  /* 38.16 YAML.deepEqual with non-string keys: different values */
  a = .table~new; a[.array~of(1, 2)] = "v1"
  b = .table~new; b[.array~of(1, 2)] = "v2"
  self~assertFalse(YAML.deepEqual(a, b), -
    "deepEqual non-string keys diff values")

  /* 38.17 YAML.deepEqual with non-string keys: different keys */
  a = .table~new; a[.array~of(1, 2)] = "v1"
  b = .table~new; b[.array~of(3, 4)] = "v1"
  self~assertFalse(YAML.deepEqual(a, b), -
    "deepEqual non-string keys diff keys")

  /* 38.18 Complex key with block value */
  yaml = "? [a, b]" || "0A"x || ":" || "0A"x || -
         "  nested: val" || "0A"x
  doc = parser~parseString(yaml)
  self~assertEquals(1, doc~items, -
    "complex key block value: one entry")
  sup = doc~supplier
  Do While sup~available
    self~assertTrue(sup~item~isA(.table), -
      "complex key block value: value is table")
    self~assertEquals("val", sup~item["nested"], -
      "complex key block value: nested")
    sup~next
  End

/*------------------------------------------------------------------------*/
/* 39. Unicode escape handling (\u / \U)                                  */
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
