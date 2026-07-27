/* extracted from yaml::test28_EscapeCharacterRoundtrips */
::routine main public
  expose parser

  /* \0 NUL */
  yaml = 'esc_nul: "before\0after"'
  doc = parser~parseString(yaml)
  self~assertEquals("before" || "00"x || "after", doc["esc_nul"], 'esc \0 parse')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["esc_nul"], doc2["esc_nul"], 'esc \0 roundtrip')

  /* \a BEL */
  yaml = 'esc_bel: "before\aafter"'
  doc = parser~parseString(yaml)
  self~assertEquals("before" || "07"x || "after", doc["esc_bel"], 'esc \a parse')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["esc_bel"], doc2["esc_bel"], 'esc \a roundtrip')

  /* \b BS */
  yaml = 'esc_bs: "before\bafter"'
  doc = parser~parseString(yaml)
  self~assertEquals("before" || "08"x || "after", doc["esc_bs"], 'esc \b parse')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["esc_bs"], doc2["esc_bs"], 'esc \b roundtrip')

  /* \t TAB */
  yaml = 'esc_tab: "before\tafter"'
  doc = parser~parseString(yaml)
  self~assertEquals("before" || "09"x || "after", doc["esc_tab"], 'esc \t parse')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["esc_tab"], doc2["esc_tab"], 'esc \t roundtrip')

  /* \v VT */
  yaml = 'esc_vt: "before\vafter"'
  doc = parser~parseString(yaml)
  self~assertEquals("before" || "0B"x || "after", doc["esc_vt"], 'esc \v parse')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["esc_vt"], doc2["esc_vt"], 'esc \v roundtrip')

  /* \f FF */
  yaml = 'esc_ff: "before\fafter"'
  doc = parser~parseString(yaml)
  self~assertEquals("before" || "0C"x || "after", doc["esc_ff"], 'esc \f parse')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["esc_ff"], doc2["esc_ff"], 'esc \f roundtrip')

  /* \r CR */
  yaml = 'esc_cr: "before\rafter"'
  doc = parser~parseString(yaml)
  self~assertEquals("before" || "0D"x || "after", doc["esc_cr"], 'esc \r parse')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["esc_cr"], doc2["esc_cr"], 'esc \r roundtrip')

  /* \e ESC */
  yaml = 'esc_esc: "before\eafter"'
  doc = parser~parseString(yaml)
  self~assertEquals("before" || "1B"x || "after", doc["esc_esc"], 'esc \e parse')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["esc_esc"], doc2["esc_esc"], 'esc \e roundtrip')

  /* \" double quote */
  yaml = 'esc_dq: "before\"after"'
  doc = parser~parseString(yaml)
  self~assertEquals('before"after', doc["esc_dq"], 'esc \" parse')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["esc_dq"], doc2["esc_dq"], 'esc \" roundtrip')

  /* \\ backslash */
  yaml = 'esc_bs2: "before\\\\after"'
  doc = parser~parseString(yaml)
  self~assertEquals("before\\after", doc["esc_bs2"], 'esc \\ parse')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["esc_bs2"], doc2["esc_bs2"], 'esc \\ roundtrip')

  /* \/ slash */
  yaml = 'esc_slash: "before\/after"'
  doc = parser~parseString(yaml)
  self~assertEquals("before/after", doc["esc_slash"], 'esc \/ parse')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["esc_slash"], doc2["esc_slash"], 'esc \/ roundtrip')

  /* \<space> */
  yaml = 'esc_space: "before\ after"'
  doc = parser~parseString(yaml)
  self~assertEquals("before after", doc["esc_space"], 'esc \  parse')
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["esc_space"], doc2["esc_space"], 'esc \  roundtrip')

  /* Combined */
  yaml = 'combined: "tab\there\abell\0nul"'
  doc = parser~parseString(yaml)
  expected = "tab" || "09"x || "here" || "07"x || "bell" || "00"x || "nul"
  self~assertEquals(expected, doc["combined"], "esc combined parse")
  yamlOut = .Yaml~toYaml(doc)
  doc2 = parser~parseString(yamlOut)
  self~assertEquals(doc["combined"], doc2["combined"], "esc combined roundtrip")

  /* XML round-trip (XSD) */
  yaml = 'esc_tab: "a\tb"' || "0A"x || -
         'esc_bel: "a\ab"' || "0A"x || -
         'esc_bs: "a\bb"'
  doc = parser~parseString(yaml)
  xml = .Yaml~yamlToXml(doc, "xsd")
  doc3 = parser~parseXml(xml)
  self~assertEquals(doc["esc_tab"], doc3["esc_tab"], "esc xml rt tab")
  self~assertEquals(doc["esc_bel"], doc3["esc_bel"], "esc xml rt bel")
  self~assertEquals(doc["esc_bs"], doc3["esc_bs"], "esc xml rt bs")

  /* XML round-trip (DTD) */
  xml2 = .Yaml~yamlToXml(doc, "dtd")
  doc4 = parser~parseXml(xml2)
  self~assertEquals(doc["esc_tab"], doc4["esc_tab"], "esc dtd rt tab")
  self~assertEquals(doc["esc_bel"], doc4["esc_bel"], "esc dtd rt bel")
  self~assertEquals(doc["esc_bs"], doc4["esc_bs"], "esc dtd rt bs")

/*------------------------------------------------------------------------*/
/* 29. Chomp preservation & embedded-newline round-trip                    */
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
