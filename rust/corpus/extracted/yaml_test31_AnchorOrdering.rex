/* extracted from yaml::test31_AnchorOrdering */
::routine main public
  expose parser

  yamlAnch = "base: &shared"        || "0A"x || -
             "  x: 10"              || "0A"x || -
             "  y: 20"              || "0A"x || -
             "ref1: *shared"        || "0A"x || -
             "ref2: *shared"
  doc = parser~parseString(yamlAnch)
  am  = parser~anchorMap

  yamlOut = .Yaml~toYaml(doc, 2, am)
  anchorPos = yamlOut~pos("&shared")
  aliasPos  = yamlOut~pos("*shared")
  self~assertTrue(anchorPos > 0, "anchor present")
  self~assertTrue(aliasPos > 0, "alias present")
  self~assertTrue(anchorPos < aliasPos, "anchor before alias")

  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "anchor order rt")

  /* Multiple anchors */
  yamlMulti = "a1: &first"          || "0A"x || -
              "  p: 1"              || "0A"x || -
              "a2: &second"         || "0A"x || -
              "  q: 2"              || "0A"x || -
              "r1: *first"          || "0A"x || -
              "r2: *second"
  doc = parser~parseString(yamlMulti)
  am  = parser~anchorMap
  yamlOut = .Yaml~toYaml(doc, 2, am)
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "multi anchor order rt")

  xml = .Yaml~yamlToXml(doc, "xsd", am)
  doc3 = parser~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc3), "anchor order xml rt")

/*------------------------------------------------------------------------*/
/* 32. Merge key reconstruction in toYaml                                 */
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
