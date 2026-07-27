/* extracted from yaml::test49_CompactBlockSequences */
::routine main public

  p = .yaml~new

  /* 49.1 Compact block sequence as mapping value (same indent) */
  yaml1 = "key:" || "0A"x || "- val1" || "0A"x || "- val2"
  doc1 = p~parseString(yaml1)
  self~assertTrue(doc1~isA(.table), "compact seq > is table")
  v1 = doc1["key"]
  self~assertTrue(v1~isA(.array), "compact seq > value is array")
  self~assertEquals(2, v1~items, "compact seq > 2 items")
  self~assertEquals("val1", v1[1], "compact seq > [1]")
  self~assertEquals("val2", v1[2], "compact seq > [2]")

  /* 49.2 Compact block sequence with multiple mapping keys */
  yaml2 = "a:" || "0A"x || "- 1" || "0A"x || "- 2" || "0A"x || "b: 3"
  doc2 = p~parseString(yaml2)
  self~assertTrue(doc2~isA(.table), "compact seq multi-key > is table")
  self~assertEquals(2, doc2~items, "compact seq multi-key > 2 entries")
  v2 = doc2["a"]
  self~assertTrue(v2~isA(.array), "compact seq multi-key > a is array")
  self~assertEquals(2, v2~items, "compact seq multi-key > a has 2 items")
  self~assertEquals(1, v2[1], "compact seq multi-key > a[1]")
  self~assertEquals(2, v2[2], "compact seq multi-key > a[2]")
  self~assertEquals(3, doc2["b"], "compact seq multi-key > b")

  /* 49.3 Comment after colon with value on next lines */
  yaml3 = "hr: # comment" || "0A"x || "  - Mark" || "0A"x || "  - Sammy"
  doc3 = p~parseString(yaml3)
  self~assertTrue(doc3~isA(.table), "comment after colon > is table")
  v3 = doc3["hr"]
  self~assertTrue(v3~isA(.array), "comment after colon > value is array")
  self~assertEquals(2, v3~items, "comment after colon > 2 items")
  self~assertEquals("Mark", v3[1], "comment after colon > [1]")
  self~assertEquals("Sammy", v3[2], "comment after colon > [2]")

  /* 49.4 Comment after colon with mapping value on next lines */
  yaml4 = "outer: # this is a comment" || "0A"x || "  inner: value"
  doc4 = p~parseString(yaml4)
  self~assertTrue(doc4~isA(.table), "comment after colon map > is table")
  v4 = doc4["outer"]
  self~assertTrue(v4~isA(.table), "comment after colon map > value is table")
  self~assertEquals("value", v4["inner"], "comment after colon map > inner")

  /* 49.5 Multiple keys with comments, compact sequences */
  yaml5 = "hr: # ranking" || "0A"x || "  - Mark" || "0A"x || "  - Sammy" || "0A"x || -
           "rbi: # ranking" || "0A"x || "  - Sammy" || "0A"x || "  - Ken"
  doc5 = p~parseString(yaml5)
  self~assertTrue(doc5~isA(.table), "multi-key comment > is table")
  self~assertEquals(2, doc5~items, "multi-key comment > 2 entries")
  self~assertTrue(doc5["hr"]~isA(.array), "multi-key comment > hr is array")
  self~assertEquals(2, doc5["hr"]~items, "multi-key comment > hr 2 items")
  self~assertTrue(doc5["rbi"]~isA(.array), "multi-key comment > rbi is array")
  self~assertEquals(2, doc5["rbi"]~items, "multi-key comment > rbi 2 items")

  /* 49.6 parseAll document count — single doc without markers */
  yaml6 = "a:" || "0A"x || "- 1" || "0A"x || "- 2" || "0A"x || "b: 3"
  docs6 = p~parseAll(yaml6)
  self~assertEquals(1, docs6~items, "parseAll > single doc no markers")

  /* 49.7 parseAll document count — explicit markers */
  yaml7 = "---" || "0A"x || "doc1" || "0A"x || "---" || "0A"x || "doc2"
  docs7 = p~parseAll(yaml7)
  self~assertEquals(2, docs7~items, "parseAll > two docs with ---")

  /* 49.8 parseAll document count — doc-end then doc-start */
  yaml8 = "doc1" || "0A"x || "..." || "0A"x || "---" || "0A"x || "doc2"
  docs8 = p~parseAll(yaml8)
  self~assertEquals(2, docs8~items, "parseAll > two docs with ... and ---")

  /* 49.9 Compact sequence at indent 0 — mapping then sequence value */
  yaml9 = "foo:" || "0A"x || "  bar" || "0A"x || "list:" || "0A"x || "- a" || "0A"x || "- b"
  doc9 = p~parseString(yaml9)
  self~assertEquals("bar", doc9["foo"], "compact seq indent 0 > foo")
  v9 = doc9["list"]
  self~assertTrue(v9~isA(.array), "compact seq indent 0 > list is array")
  self~assertEquals(2, v9~items, "compact seq indent 0 > list 2 items")

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
