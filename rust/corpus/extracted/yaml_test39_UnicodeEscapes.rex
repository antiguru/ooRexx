/* extracted from yaml::test39_UnicodeEscapes */
::routine main public
  expose parser

  /* 39.1 \u 2-byte UTF-8 (é = U+00E9 → C3 A9) */
  yaml = 'val: "\u00E9"'
  doc = parser~parseString(yaml)
  self~assertEquals("C3A9", doc["val"]~c2x, '\u 2-byte UTF-8 (é)')

  /* 39.2 \u 1-byte ASCII (A = U+0041 → 41) */
  yaml = 'val: "\u0041"'
  doc = parser~parseString(yaml)
  self~assertEquals("A", doc["val"], '\u 1-byte ASCII (A)')

  /* 39.3 \u 3-byte UTF-8 (世 = U+4E16 → E4 B8 96) */
  yaml = 'val: "\u4E16"'
  doc = parser~parseString(yaml)
  self~assertEquals("E4B896", doc["val"]~c2x, '\u 3-byte UTF-8 (世)')

  /* 39.4 \U 4-byte UTF-8 (U+1F600 → F0 9F 98 80) */
  yaml = 'val: "\U0001F600"'
  doc = parser~parseString(yaml)
  self~assertEquals("F09F9880", doc["val"]~c2x, '\U 4-byte UTF-8 (U+1F600)')

  /* 39.5 \u null char (U+0000 → 00) */
  yaml = 'val: "\u0000"'
  doc = parser~parseString(yaml)
  self~assertEquals("00", doc["val"]~c2x, '\u null char (U+0000)')

  /* 39.6 \u 2-byte boundary (U+0080 → C2 80) */
  yaml = 'val: "\u0080"'
  doc = parser~parseString(yaml)
  self~assertEquals("C280", doc["val"]~c2x, '\u 2-byte boundary (U+0080)')

  /* 39.7 \u 3-byte boundary (U+0800 → E0 A0 80) */
  yaml = 'val: "\u0800"'
  doc = parser~parseString(yaml)
  self~assertEquals("E0A080", doc["val"]~c2x, '\u 3-byte boundary (U+0800)')

  /* 39.8 \U 4-byte boundary (U+10000 → F0 90 80 80) */
  yaml = 'val: "\U00010000"'
  doc = parser~parseString(yaml)
  self~assertEquals("F0908080", doc["val"]~c2x, '\U 4-byte boundary (U+10000)')

  /* 39.9 Mixed \u with other escapes */
  yaml = 'val: "caf\u00E9\tbr\u00FBl\u00E9e"'
  doc = parser~parseString(yaml)
  expected = "caf" || "C3A9"x || "09"x || "br" || "C3BB"x || "l" || "C3A9"x || "e"
  self~assertEquals(expected~c2x, doc["val"]~c2x, '\u mixed with \t')

  /* 39.10 Multiple \u in one string */
  yaml = 'val: "\u00C0\u00C1\u00C2"'
  doc = parser~parseString(yaml)
  expected = "C380"x || "C381"x || "C382"x
  self~assertEquals(expected~c2x, doc["val"]~c2x, 'multiple \u in string')

  /* 39.11 Unescaped \u round-trip */
  yaml = 'val: "\u00E9"'
  doc = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals("C3A9", doc2["val"]~c2x, '\u round-trip (unescaped)')

  /* 39.12 Legacy mode: \u stored literally */
  parserLegacy = .Yaml~new(.false)
  yaml = 'val: "\u00E9"'
  doc = parserLegacy~parseString(yaml)
  self~assertEquals("\u00E9", doc["val"], '\u legacy mode literal')

  /* 39.13 Legacy mode: \U stored literally */
  yaml = 'val: "\U0001F600"'
  doc = parserLegacy~parseString(yaml)
  self~assertEquals("\U0001F600", doc["val"], '\U legacy mode literal')

  /* 39.14 Legacy mode round-trip */
  yaml = 'val: "\u00E9"'
  doc = parserLegacy~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parserLegacy~parseString(yamlOut)
  self~assertEquals("\u00E9", doc2["val"], '\u legacy round-trip')

  /* 39.15 \u in mapping key (unescaped) */
  yaml = '"\u00E9": value'
  doc = parser~parseString(yaml)
  self~assertEquals("value", doc["C3A9"x], '\u in mapping key')

  /* 39.16 \u in sequence items (unescaped) */
  yaml = '- "\u00E9"' || "0A"x || '- "\u4E16"'
  doc = parser~parseString(yaml)
  self~assertEquals("C3A9", doc[1]~c2x, '\u in seq item 1')
  self~assertEquals("E4B896", doc[2]~c2x, '\u in seq item 2')

  /* 39.17 \u XML round-trip (unescaped) */
  yaml = 'val: "\u00E9"'
  doc = parser~parseString(yaml)
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc2 = parser~parseXml(xml)
  self~assertEquals("C3A9", doc2["val"]~c2x, '\u XML round-trip')

  /* 39.18 \U in flow collection */
  yaml = 'items: ["\U0001F600", "\u00E9"]'
  doc = parser~parseString(yaml)
  self~assertEquals("F09F9880", doc["items"][1]~c2x, '\U in flow seq')
  self~assertEquals("C3A9", doc["items"][2]~c2x, '\u in flow seq')

/*------------------------------------------------------------------------*/
/* 40. Complex keys in XML round-trip                                     */
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
