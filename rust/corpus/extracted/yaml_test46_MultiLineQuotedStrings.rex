/* extracted from yaml::test46_MultiLineQuotedStrings */
::routine main public

  p = .Yaml~new

  /* 46.1  Double-quoted: fold across two lines */
  yaml1 = '---'                                || "0A"x || -
          'key: "hello'                        || "0A"x || -
          '  world"'                           || "0A"x
  doc1 = p~parseString(yaml1)
  self~assertEquals("hello world", doc1["key"], "dq fold two lines")

  /* 46.2  Single-quoted: fold across two lines */
  yaml2 = '---'                                || "0A"x || -
          "key: 'hello"                        || "0A"x || -
          "  world'"                           || "0A"x
  doc2 = p~parseString(yaml2)
  self~assertEquals("hello world", doc2["key"], "sq fold two lines")

  /* 46.3  Double-quoted: three continuation lines */
  yaml3 = '---'                                || "0A"x || -
          'key: "line one'                     || "0A"x || -
          '  line two'                         || "0A"x || -
          '  line three"'                      || "0A"x
  doc3 = p~parseString(yaml3)
  self~assertEquals("line one line two line three", doc3["key"], "dq three lines")

  /* 46.4  Double-quoted: blank line → literal newline */
  yaml4 = '---'                                || "0A"x || -
          'key: "first'                        || "0A"x || -
          ''                                   || "0A"x || -
          '  second"'                          || "0A"x
  doc4 = p~parseString(yaml4)
  self~assertEquals("first" || "0A"x || "second", doc4["key"], "dq blank → newline")

  /* 46.5  Single-quoted: blank line → literal newline */
  yaml5 = '---'                                || "0A"x || -
          "key: 'first"                        || "0A"x || -
          ''                                   || "0A"x || -
          "  second'"                          || "0A"x
  doc5 = p~parseString(yaml5)
  self~assertEquals("first" || "0A"x || "second", doc5["key"], "sq blank → newline")

  /* 46.6  Double-quoted: escape-newline joins without space */
  yaml6 = '---'                                || "0A"x || -
          'key: "no\'                          || "0A"x || -
          '  space"'                           || "0A"x
  doc6 = p~parseString(yaml6)
  self~assertEquals("nospace", doc6["key"], "dq escape-newline")

  /* 46.7  Double-quoted: escape in multi-line content */
  yaml7 = '---'                                || "0A"x || -
          'key: "has \t tab'                   || "0A"x || -
          '  and more"'                        || "0A"x
  doc7 = p~parseString(yaml7)
  self~assertEquals("has " || "09"x || " tab and more", doc7["key"], "dq escape multiline")

  /* 46.8  Multi-line dq in sequence item */
  yaml8 = '---'                                || "0A"x || -
          'items:'                             || "0A"x || -
          '  - "multi'                         || "0A"x || -
          '    line"'                          || "0A"x || -
          '  - plain'                          || "0A"x
  doc8 = p~parseString(yaml8)
  self~assertEquals("multi line", doc8["items"][1], "dq in seq item")
  self~assertEquals("plain", doc8["items"][2], "plain after dq in seq")

  /* 46.9  Multi-line sq in sequence item */
  yaml9 = '---'                                || "0A"x || -
          'items:'                             || "0A"x || -
          "  - 'single"                        || "0A"x || -
          "    line'"                          || "0A"x || -
          '  - other'                          || "0A"x
  doc9 = p~parseString(yaml9)
  self~assertEquals("single line", doc9["items"][1], "sq in seq item")
  self~assertEquals("other", doc9["items"][2], "plain after sq in seq")

  /* 46.10  Multi-line dq as mapping value with next key */
  yaml10 = '---'                               || "0A"x || -
           'k1: "multi'                        || "0A"x || -
           '  line"'                           || "0A"x || -
           'k2: normal'                        || "0A"x
  doc10 = p~parseString(yaml10)
  self~assertEquals("multi line", doc10["k1"], "dq with next key")
  self~assertEquals("normal", doc10["k2"], "next key after dq")

  /* 46.11  Single-quoted with doubled quote in multi-line */
  yaml11 = '---'                               || "0A"x || -
           "key: 'it''s"                       || "0A"x || -
           "  multi'"                          || "0A"x
  doc11 = p~parseString(yaml11)
  self~assertEquals("it's multi", doc11["key"], "sq doubled in multiline")

  /* 46.12  Two consecutive blank lines → two newlines */
  yaml12 = '---'                               || "0A"x || -
           'key: "first'                       || "0A"x || -
           ''                                  || "0A"x || -
           ''                                  || "0A"x || -
           '  second"'                         || "0A"x
  doc12 = p~parseString(yaml12)
  self~assertEquals("first" || "0A0A"x || "second", doc12["key"], "dq two blanks")

  /* 46.13  Multi-line dq as block scalar (blockScalarOrPlain path) */
  yaml13 = '---'                               || "0A"x || -
           'parent:'                           || "0A"x || -
           '  "multi'                          || "0A"x || -
           '  line block"'                     || "0A"x
  doc13 = p~parseString(yaml13)
  self~assertEquals("multi line block", doc13["parent"], "dq as block scalar")

  /* 46.14  YAML round-trip */
  yaml_rt = .Yaml~toYaml(doc4)
  doc_rt = p~parseString(yaml_rt)
  self~assertEquals(doc4["key"], doc_rt["key"], "dq blank YAML rt")

  /* 46.15  XML round-trip */
  xml = .Yaml~yamlToXml(doc4)
  doc_x = p~parseXml(xml)
  self~assertEquals(doc4["key"], doc_x["key"], "dq blank XML rt")

  /* 46.16  YAML round-trip for escape-newline */
  yaml_frt = .Yaml~toYaml(doc6)
  doc_frt = p~parseString(yaml_frt)
  self~assertEquals("nospace", doc_frt["key"], "escape-newline YAML rt")

  /* 46.17  YAML round-trip for multi-line sequence */
  yaml_hrt = .Yaml~toYaml(doc8)
  doc_hrt = p~parseString(yaml_hrt)
  self~assertEquals("multi line", doc_hrt["items"][1], "dq seq YAML rt")

  /* 46.18  Single-quoted: two consecutive blank lines → two newlines */
  yaml18 = '---'                               || "0A"x || -
           "key: 'first"                       || "0A"x || -
           ''                                  || "0A"x || -
           ''                                  || "0A"x || -
           "  second'"                         || "0A"x
  doc18 = p~parseString(yaml18)
  self~assertEquals("first" || "0A0A"x || "second", doc18["key"], "sq two blank lines")

  /* 46.19  Multi-line sq as standalone block scalar */
  yaml19 = '---'                               || "0A"x || -
           'parent:'                           || "0A"x || -
           "  'multi"                          || "0A"x || -
           "  line block'"                     || "0A"x
  doc19 = p~parseString(yaml19)
  self~assertEquals("multi line block", doc19["parent"], "sq as block scalar")

/*========================================================================*/
/* Group 47 — Multi-line flow collections (P5)                           */
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
