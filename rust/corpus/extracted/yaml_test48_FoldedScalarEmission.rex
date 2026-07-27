/* extracted from yaml::test48_FoldedScalarEmission */
::routine main public

  p = .Yaml~new

  /* 48.1  Single-line content emits as > (clip) */
  val1 = "this is one line" || "0A"x
  doc1 = .table~new; doc1["text"] = val1
  yaml1 = .Yaml~toYaml(doc1)
  self~assertTrue(yaml1~pos(">") > 0, "folded > emitted for single-line")
  self~assertTrue(yaml1~pos("|") = 0, "folded no literal for single-line")
  doc1r = p~parseString(yaml1)
  self~assertEquals(val1, doc1r["text"], "folded > clip roundtrip")

  /* 48.2  Single-line content with >+ (keep, 2 trailing NLs) */
  val2 = "trailing newlines kept" || "0A"x || "0A"x
  doc2 = .table~new; doc2["keep"] = val2
  yaml2 = .Yaml~toYaml(doc2)
  self~assertTrue(yaml2~pos(">+") > 0, "folded >+ emitted")
  doc2r = p~parseString(yaml2)
  self~assertEquals(val2, doc2r["keep"], "folded >+ roundtrip")

  /* 48.3  Multi-line content stays as | (literal) */
  val3 = "line one" || "0A"x || "line two" || "0A"x
  doc3 = .table~new; doc3["code"] = val3
  yaml3 = .Yaml~toYaml(doc3)
  self~assertTrue(yaml3~pos("|") > 0, "literal | for multi-line")
  doc3r = p~parseString(yaml3)
  self~assertEquals(val3, doc3r["code"], "literal | multi-line roundtrip")

  /* 48.4  Content starting with space stays as | */
  val4 = " indented start" || "0A"x
  doc4 = .table~new; doc4["indent"] = val4
  yaml4 = .Yaml~toYaml(doc4)
  self~assertTrue(yaml4~pos("|") > 0, "literal | for space-start content")
  doc4r = p~parseString(yaml4)
  self~assertEquals(val4, doc4r["indent"], "literal | space-start roundtrip")

  /* 48.5  Long single-line wraps to multiple folded lines */
  val5 = copies("word ", 20)~strip || "0A"x
  doc5 = .table~new; doc5["long"] = val5
  yaml5 = .Yaml~toYaml(doc5)
  self~assertTrue(yaml5~pos(">") > 0, "folded > for long line")
  /* The folded body should have more than one indented line */
  bodyLines = 0
  Do line Over yaml5~makeArray("0A"x)
    If line~left(2) == "  " Then bodyLines = bodyLines + 1
  End
  self~assertTrue(bodyLines > 1, "folded long line wrapped")
  doc5r = p~parseString(yaml5)
  self~assertEquals(val5, doc5r["long"], "folded long line roundtrip")

  /* 48.6  Folded in sequence context */
  val6 = "sequence item text" || "0A"x
  doc6 = .array~of(val6, "plain")
  yaml6 = .Yaml~toYaml(doc6)
  self~assertTrue(yaml6~pos(">") > 0, "folded > in sequence")
  doc6r = p~parseString(yaml6)
  self~assertEquals(val6, doc6r[1], "folded > in seq roundtrip")

  /* 48.7  Folded in nested mapping */
  inner7 = .table~new; inner7["desc"] = "a description" || "0A"x
  doc7 = .table~new; doc7["outer"] = inner7
  yaml7 = .Yaml~toYaml(doc7)
  self~assertTrue(yaml7~pos(">") > 0, "folded > nested mapping")
  doc7r = p~parseString(yaml7)
  self~assertEquals(inner7["desc"], doc7r["outer"]["desc"], -
    "folded > nested roundtrip")

  /* 48.8  String not ending with NL uses double-quoted (unchanged) */
  val8 = "no trailing" || "0A"x || "newline"
  doc8 = .table~new; doc8["dq"] = val8
  yaml8 = .Yaml~toYaml(doc8)
  self~assertTrue(yaml8~pos('"') > 0, "double-quoted no trailing NL")
  doc8r = p~parseString(yaml8)
  self~assertEquals(val8, doc8r["dq"], "double-quoted no trailing NL roundtrip")

  /* 48.9  Folded >+ with 3 trailing NLs */
  val9 = "three trailing" || "0A"x || "0A"x || "0A"x
  doc9 = .table~new; doc9["many"] = val9
  yaml9 = .Yaml~toYaml(doc9)
  self~assertTrue(yaml9~pos(">+") > 0, "folded >+ three trailing")
  doc9r = p~parseString(yaml9)
  self~assertEquals(val9, doc9r["many"], "folded >+ three trailing roundtrip")

  /* 48.10 XML round-trip for folded content (XSD and DTD) */
  xml10a = .Yaml~yamlToXml(doc1)
  doc10a = p~parseXml(xml10a)
  self~assertEquals(val1, doc10a["text"], "folded > XML xsd roundtrip")

  xml10b = .Yaml~yamlToXml(doc2, "dtd")
  doc10b = p~parseXml(xml10b)
  self~assertEquals(val2, doc10b["keep"], "folded >+ XML dtd roundtrip")

  /* 48.11 Mixed block scalar types round-trip */
  doc11 = .table~new
  doc11["folded_single"]  = "one line of text" || "0A"x
  doc11["folded_keep"]    = "kept" || "0A"x || "0A"x
  doc11["literal_multi"]  = "a" || "0A"x || "b" || "0A"x
  doc11["literal_space"]  = " starts with space" || "0A"x
  doc11["double_quoted"]  = "no" || "0A"x || "trailing"
  yaml11 = .Yaml~toYaml(doc11)
  doc11r = p~parseString(yaml11)
  self~assertTrue(YAML.deepEqual(doc11, doc11r), "mixed block scalars roundtrip")

  /* 48.12 Tagged folded scalar round-trip */
  pt = .Yaml~new("preserveTags", .true)
  doc12 = .table~new
  doc12["desc"] = .YamlTagged~new("!custom", "a single line" || "0A"x)
  yaml12 = .Yaml~toYaml(doc12)
  self~assertTrue(yaml12~pos("!custom >") > 0, "tagged folded > on same line")
  doc12r = pt~parseString(yaml12)
  self~assertEquals("!custom", doc12r["desc"]~tag, -
    "tagged folded > tag preserved")
  self~assertEquals(doc12["desc"]~value, doc12r["desc"]~value, -
    "tagged folded > roundtrip")

  /* 48.13 Tagged literal scalar round-trip */
  doc13 = .table~new
  doc13["code"] = .YamlTagged~new("!code", "a" || "0A"x || "b" || "0A"x)
  yaml13 = .Yaml~toYaml(doc13)
  self~assertTrue(yaml13~pos("!code |") > 0, "tagged literal | on same line")
  doc13r = pt~parseString(yaml13)
  self~assertEquals(doc13["code"]~value, doc13r["code"]~value, -
    "tagged literal | roundtrip")

  /* 48.14 Tagged folded >+ (keep) round-trip */
  doc14 = .table~new
  doc14["k"] = .YamlTagged~new("!keep", "kept" || "0A"x || "0A"x)
  yaml14 = .Yaml~toYaml(doc14)
  self~assertTrue(yaml14~pos("!keep >+") > 0, "tagged folded >+ on same line")
  doc14r = pt~parseString(yaml14)
  self~assertEquals(doc14["k"]~value, doc14r["k"]~value, -
    "tagged folded >+ roundtrip")

  /* 48.15 Content exactly at wrap boundary (76 chars) */
  val15 = copies("x", 76) || "0A"x
  doc15 = .table~new; doc15["exact"] = val15
  yaml15 = .Yaml~toYaml(doc15)
  doc15r = p~parseString(yaml15)
  self~assertEquals(val15, doc15r["exact"], -
    "folded > exact boundary roundtrip")

  /* 48.16 Single very long word (no spaces, 200 chars) */
  val16 = copies("abcdefghij", 20) || "0A"x
  doc16 = .table~new; doc16["long"] = val16
  yaml16 = .Yaml~toYaml(doc16)
  doc16r = p~parseString(yaml16)
  self~assertEquals(val16, doc16r["long"], -
    "folded > long word no spaces roundtrip")

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
