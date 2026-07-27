/* extracted from json_02::test_legible_vs_minimized_roundtrip */
::routine main public
  expose json
  dir = .directory~new
  dir["name"] = "test"
  dir["items"] = .array~of(1, 2, 3)
  inner = .directory~new
  inner["flag"] = .JsonBoolean~true
  dir["config"] = inner

  legibleJson = json~toJSON(dir, .true)
  decoded = json~fromJSON(legibleJson)
  minimized = json~toJSON(decoded)
  decoded2 = json~fromJSON(minimized)
  self~assertTrue(json.deepEqual(decoded, decoded2), "legible vs minimized round-trip")


/*============================================================================*/
/*  json_class_methods_test  — group 19: class methods                        */
/*============================================================================*/


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
