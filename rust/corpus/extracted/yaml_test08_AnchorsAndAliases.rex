/* extracted from yaml::test08_AnchorsAndAliases */
::routine main public
  expose parser

  yaml = "defaults: &defs"     || "0A"x || -
         "  adapter: postgres"  || "0A"x || -
         "  host: localhost"    || "0A"x || -
         "development:"         || "0A"x || -
         "  <<: *defs"          || "0A"x || -
         "  database: myapp_dev"

  doc = parser~parseString(yaml)
  dev = doc["development"]
  self~assertEquals("postgres", dev["adapter"], "merge adapter")
  self~assertEquals("localhost", dev["host"], "merge host")
  self~assertEquals("myapp_dev", dev["database"], "own key")

/*------------------------------------------------------------------------*/
/* 9. Sequence of mappings                                                */
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
