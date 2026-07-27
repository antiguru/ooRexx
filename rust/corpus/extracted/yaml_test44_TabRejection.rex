/* extracted from yaml::test44_TabRejection */
::routine main public

  /* 44.1 Tab at start of indentation should raise an error */
  yaml1 = "key:" || "0A"x || "09"x || "value: bad"
  raised1 = .false
  signal on syntax name tab44_1
  .Yaml~new~parseString(yaml1)
  signal tab44_1done
  tab44_1:
  raised1 = .true
  tab44_1done:
  self~assertTrue(raised1, "tab indent raises error")

  /* 44.2 Tab after spaces should also raise */
  yaml2 = "key:" || "0A"x || "  " || "09"x || "value: bad"
  raised2 = .false
  signal on syntax name tab44_2
  .Yaml~new~parseString(yaml2)
  signal tab44_2done
  tab44_2:
  raised2 = .true
  tab44_2done:
  self~assertTrue(raised2, "tab after spaces raises error")

  /* 44.3 Tab in content (not indentation) should be fine */
  yaml3 = 'key: "value' || "09"x || 'with tab"'
  doc3 = .Yaml~new~parseString(yaml3)
  self~assertEquals("value" || "09"x || "with tab", doc3["key"], "tab in content OK")

/*========================================================================*/
/* Group 45 — Explicit key ? with key on next line (P10)                 */
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
