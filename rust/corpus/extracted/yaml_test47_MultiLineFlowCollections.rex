/* extracted from yaml::test47_MultiLineFlowCollections */
::routine main public

  p = .Yaml~new

  /* 47.1  Flow sequence spanning lines as mapping value */
  yaml1 = '---'                                || "0A"x || -
          'key: [a,'                           || "0A"x || -
          '  b, c]'                            || "0A"x
  doc1 = p~parseString(yaml1)
  self~assertEquals(3, doc1["key"]~items, "flow seq ml: items")
  self~assertEquals("a", doc1["key"][1], "flow seq ml: [1]")
  self~assertEquals("c", doc1["key"][3], "flow seq ml: [3]")

  /* 47.2  Flow mapping spanning lines as mapping value */
  yaml2 = '---'                                || "0A"x || -
          'key: {a: 1,'                        || "0A"x || -
          '  b: 2}'                            || "0A"x
  doc2 = p~parseString(yaml2)
  self~assertEquals(1, doc2["key"]["a"], "flow map ml: a")
  self~assertEquals(2, doc2["key"]["b"], "flow map ml: b")

  /* 47.3  Flow seq in sequence item */
  yaml3 = '---'                                || "0A"x || -
          'items:'                             || "0A"x || -
          '  - [x,'                            || "0A"x || -
          '    y]'                             || "0A"x || -
          '  - plain'                          || "0A"x
  doc3 = p~parseString(yaml3)
  self~assertEquals(2, doc3["items"][1]~items, "flow seq in seq: items")
  self~assertEquals("x", doc3["items"][1][1], "flow seq in seq: [1]")
  self~assertEquals("plain", doc3["items"][2], "flow seq in seq: next")

  /* 47.4  Flow map in sequence item */
  yaml4 = '---'                                || "0A"x || -
          'items:'                             || "0A"x || -
          '  - {p: 1,'                         || "0A"x || -
          '    q: 2}'                          || "0A"x
  doc4 = p~parseString(yaml4)
  self~assertEquals(1, doc4["items"][1]["p"], "flow map in seq: p")
  self~assertEquals(2, doc4["items"][1]["q"], "flow map in seq: q")

  /* 47.5  Nested multi-line flow */
  yaml5 = '---'                                || "0A"x || -
          'key: {outer: [1,'                   || "0A"x || -
          '  2, 3],'                           || "0A"x || -
          '  other: ok}'                       || "0A"x
  doc5 = p~parseString(yaml5)
  self~assertEquals(3, doc5["key"]["outer"]~items, "nested ml: items")
  self~assertEquals("ok", doc5["key"]["other"], "nested ml: other")

  /* 47.6  Flow collection in nested mapping value */
  yaml6 = '---'                                || "0A"x || -
          'items:'                             || "0A"x || -
          '  - name: test'                     || "0A"x || -
          '    tags: [a,'                      || "0A"x || -
          '      b]'                           || "0A"x
  doc6 = p~parseString(yaml6)
  self~assertEquals(2, doc6["items"][1]["tags"]~items, "flow in nested: items")
  self~assertEquals("a", doc6["items"][1]["tags"][1], "flow in nested: [1]")
  self~assertEquals("b", doc6["items"][1]["tags"][2], "flow in nested: [2]")

  /* 47.7  YAML round-trip */
  yaml_rt = .Yaml~toYaml(doc1)
  doc_rt = p~parseString(yaml_rt)
  self~assertEquals(3, doc_rt["key"]~items, "flow seq ml YAML rt")

  /* 47.8  XML round-trip */
  xml = .Yaml~yamlToXml(doc2)
  doc_x = p~parseXml(xml)
  self~assertEquals(1, doc_x["key"]["a"], "flow map ml XML rt: a")
  self~assertEquals(2, doc_x["key"]["b"], "flow map ml XML rt: b")

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
