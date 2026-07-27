/* extracted from yaml::test34_FlowMappings */
::routine main public
  expose parser

  /* Small nested mapping emitted in flow style */
  doc = .table~new
  inner = .table~new
  inner["x"] = 10; inner["y"] = 20
  doc["point"] = inner
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos("{") > 0, "flow map emitted for small nested")

  /* Flow map round-trip */
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(10, doc2["point"]["x"], "flow map roundtrip x")
  self~assertEquals(20, doc2["point"]["y"], "flow map roundtrip y")

  /* Large mappings stay block */
  doc3 = .table~new
  big = .table~new
  Do i = 1 To 6
    big["key" || i] = "value" || i
  End
  doc3["big"] = big
  yamlOut3 = .Yaml~toYaml(doc3)
  self~assertTrue(yamlOut3~pos("{key") = 0, "big map stays block")

  /* Top-level mapping stays block */
  doc4 = .table~new
  doc4["a"] = 1; doc4["b"] = 2
  yamlOut4 = .Yaml~toYaml(doc4)
  self~assertTrue(yamlOut4~left(1) \== "{", "top-level map stays block")

  /* Mapping with nested collections stays block */
  doc5 = .table~new
  inner2 = .table~new
  inner2["a"] = .array~of(1, 2)
  doc5["nested"] = inner2
  yamlOut5 = .Yaml~toYaml(doc5)
  self~assertTrue(yamlOut5~pos("{a:") = 0, "nested collection stays block")

  /* Sequence of small maps — flow emitted, round-trip */
  doc6 = .table~new
  arr = .array~new
  m1 = .table~new; m1["name"] = "Alice"; m1["age"] = 30
  m2 = .table~new; m2["name"] = "Bob"; m2["age"] = 25
  arr~append(m1); arr~append(m2)
  doc6["people"] = arr
  yamlOut6 = .Yaml~toYaml(doc6)
  doc6b = parser~parseString(yamlOut6)
  self~assertEquals("Alice", doc6b["people"][1]["name"], -
    "flow map in seq roundtrip name")
  self~assertEquals(25, doc6b["people"][2]["age"], -
    "flow map in seq roundtrip age")

/*------------------------------------------------------------------------*/
/* 35. Directives (%YAML, %TAG)                                           */
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
