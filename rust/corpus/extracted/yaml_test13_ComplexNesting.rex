/* extracted from yaml::test13_ComplexNesting */
::routine main public
  expose parser

  yaml = "servers:"                 || "0A"x || -
         "  - name: web1"           || "0A"x || -
         "    roles:"               || "0A"x || -
         "      - frontend"         || "0A"x || -
         "      - api"              || "0A"x || -
         "    config:"              || "0A"x || -
         "      port: 8080"         || "0A"x || -
         "      ssl: true"          || "0A"x || -
         "  - name: db1"            || "0A"x || -
         "    roles:"               || "0A"x || -
         "      - database"

  doc = parser~parseString(yaml)
  servers = doc["servers"]
  self~assertEquals(2, servers~items, "server count")
  self~assertEquals("web1", servers[1]["name"], "server1 name")
  roles = servers[1]["roles"]
  self~assertEquals(2, roles~items, "server1 roles")
  self~assertEquals("frontend", roles[1], "server1 role 1")
  cfg = servers[1]["config"]
  self~assertEquals(8080, cfg["port"], "server1 port")
  self~assertEquals(1, cfg["ssl"], "server1 ssl")
  self~assertEquals(1, servers[2]["roles"]~items, "server2 role cnt")

/*------------------------------------------------------------------------*/
/* 14. Nested flow collections                                            */
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
