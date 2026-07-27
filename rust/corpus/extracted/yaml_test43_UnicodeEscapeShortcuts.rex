/* extracted from yaml::test43_UnicodeEscapeShortcuts */
::routine main public

  yaml = '---'                                || "0A"x || -
         'next_line: "hello\Nworld"'          || "0A"x || -
         'nbsp: "non\_breaking"'              || "0A"x || -
         'line_sep: "line\Lsep"'              || "0A"x || -
         'para_sep: "para\Psep"'              || "0A"x

  p = .Yaml~new
  doc = p~parseString(yaml)

  /* 43.1 \N -> U+0085 (UTF-8: C2 85) */
  self~assertEquals("hello" || "C285"x || "world", doc["next_line"], "\N escape")

  /* 43.2 \_ -> U+00A0 (UTF-8: C2 A0) */
  self~assertEquals("non" || "C2A0"x || "breaking", doc["nbsp"], "\_ escape")

  /* 43.3 \L -> U+2028 (UTF-8: E2 80 A8) */
  self~assertEquals("line" || "E280A8"x || "sep", doc["line_sep"], "\L escape")

  /* 43.4 \P -> U+2029 (UTF-8: E2 80 A9) */
  self~assertEquals("para" || "E280A9"x || "sep", doc["para_sep"], "\P escape")

  /* 43.5-8 YAML round-trip */
  yaml_rt = .Yaml~toYaml(doc)
  doc_rt = .Yaml~new~parseString(yaml_rt)
  self~assertEquals(doc["next_line"], doc_rt["next_line"], "\N YAML rt")
  self~assertEquals(doc["nbsp"], doc_rt["nbsp"], "\_ YAML rt")
  self~assertEquals(doc["line_sep"], doc_rt["line_sep"], "\L YAML rt")
  self~assertEquals(doc["para_sep"], doc_rt["para_sep"], "\P YAML rt")

  /* 43.9-12 XML round-trip (XSD) */
  xml = .Yaml~yamlToXml(doc)
  doc_x = p~parseXml(xml)
  self~assertEquals(doc["next_line"], doc_x["next_line"], "\N XML rt")
  self~assertEquals(doc["nbsp"], doc_x["nbsp"], "\_ XML rt")
  self~assertEquals(doc["line_sep"], doc_x["line_sep"], "\L XML rt")
  self~assertEquals(doc["para_sep"], doc_x["para_sep"], "\P XML rt")

/*========================================================================*/
/* Group 44 — Tab rejection in indentation (P7)                          */
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
