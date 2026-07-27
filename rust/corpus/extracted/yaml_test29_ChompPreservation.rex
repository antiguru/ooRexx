/* extracted from yaml::test29_ChompPreservation */
::routine main public
  expose parser

  /* 29a. Clip chomp (|) */
  yaml = "clip: |" || "0A"x || "  line1" || "0A"x || "  line2"
  doc = parser~parseString(yaml)
  clipVal = doc["clip"]
  self~assertEquals("line1" || "0A"x || "line2" || "0A"x, clipVal, "clip parse")
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos("|") > 0, "clip emits |")
  self~assertTrue(yamlOut~pos("|+") = 0, "clip not |+")
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(clipVal, doc2["clip"], "clip roundtrip")

  /* 29b. Strip chomp (|-) */
  yaml = "strip: |-" || "0A"x || "  line1" || "0A"x || "  line2"
  doc = parser~parseString(yaml)
  stripVal = doc["strip"]
  self~assertEquals("line1" || "0A"x || "line2", stripVal, "strip parse")
  self~assertTrue(stripVal~right(1) \== "0A"x, "strip no trailing NL")
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos('"') > 0, 'strip emits dquoted')
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(stripVal, doc2["strip"], "strip roundtrip")

  /* 29c. Keep chomp (|+) */
  yaml = "keep: |+" || "0A"x || "  line1" || "0A"x || "  line2" || "0A"x || "0A"x || "0A"x
  doc = parser~parseString(yaml)
  keepVal = doc["keep"]
  self~assertTrue(keepVal~right(2) == "0A0A"x, "keep trailing NLs")
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos("|+") > 0, "keep emits |+")
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(keepVal, doc2["keep"], "keep roundtrip")

  /* 29d. Embedded newline in double-quoted (no trailing NL) */
  yaml = 'embedded: "line1\nline2"'
  doc = parser~parseString(yaml)
  embVal = doc["embedded"]
  self~assertEquals("line1" || "0A"x || "line2", embVal, "embedded parse")
  self~assertTrue(embVal~right(1) \== "0A"x, "embedded no trailing NL")
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos('"') > 0, "embedded emits dquoted")
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(embVal, doc2["embedded"], "embedded roundtrip")

  /* 29e. Embedded newline with trailing NL — should use | */
  yaml = 'mixed: "line1\nline2\n"'
  doc = parser~parseString(yaml)
  mixVal = doc["mixed"]
  self~assertEquals("line1" || "0A"x || "line2" || "0A"x, mixVal, "mixed parse")
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos("|") > 0, "mixed emits |")
  self~assertTrue(yamlOut~pos("|+") = 0, "mixed not |+")
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(mixVal, doc2["mixed"], "mixed roundtrip")

  /* 29f. Full round-trip for all chomp variants */
  yaml = "clip: |"  || "0A"x || "  A" || "0A"x || "  B" || "0A"x || -
         "strip: |-" || "0A"x || "  C" || "0A"x || "  D" || "0A"x || -
         "keep: |+" || "0A"x || "  E" || "0A"x || "  F" || "0A"x || "0A"x || "0A"x || -
         'embed: "G\nH"'
  doc = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "all chomp deepEqual")

  /* 29g. XML round-trips */
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc3 = parser~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc3), "chomp xml xsd rt")

  xml2 = .Yaml~yamlToXml(doc, "dtd")
  doc4 = parser~parseXml(xml2)
  self~assertTrue(YAML.deepEqual(doc, doc4), "chomp xml dtd rt")

  /* 29h. Folded (>) round-trip */
  yaml = "folded: >" || "0A"x || "  this is" || "0A"x || "  one line"
  doc = parser~parseString(yaml)
  foldVal = doc["folded"]
  self~assertTrue(foldVal~right(1) == "0A"x, "folded trailing NL")
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(foldVal, doc2["folded"], "folded roundtrip")

/*------------------------------------------------------------------------*/
/* 30. Flow style emission                                                */
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
