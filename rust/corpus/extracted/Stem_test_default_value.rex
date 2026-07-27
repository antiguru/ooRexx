/* extracted from Stem::test_default_value */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem o1 o2

         /* explicitly created stem object      */
  s=clz~new          -- no default value given, hence empty string ""
  self~assertSame("", s~at)
  self~assertSame("1", s~at("1"))
  self~assertSame("aha", s~at("aha"))
  self~assertSame("1.A.1", s~at(1, "A", 1))

  self~assertSame("", s~"[]")
  self~assertSame("1", s~"[]"("1"))
  self~assertSame("aha", s~"[]"("aha"))
  self~assertSame("1.A.1", s~"[]"(1, "A", 1))

       /* implicitly created stem object, using stem name as default value  */
  self~assertSame("A.", a.~at)
  self~assertSame("A.1", a.~at("1"))
  self~assertSame("A.aha", a.~at("aha"))
  self~assertSame("A.1.A.1", a.~at(1, "A", 1))

  self~assertSame("A.", a.~"[]")
  self~assertSame("A.1", a.~"[]"("1"))
  self~assertSame("A.aha", a.~"[]"("aha"))
  self~assertSame("A.1.A.1", a.~"[]"(1, "A", 1))

      /* explicitly created stem object, changing default values */
  defVal1="DefaultValue1"
  defVal2="DefaultValue2"
  s=clz~new(defVal1)

  self~assertSame(defVal1, s~at)
  self~assertSame(defVal1, s~"[]")

  self~assertEquals(0, s~items)
  s~put("1", "1")
  self~assertEquals(1, s~items)

         /* resetting default value, will empty stem  */
  s~put(defVal2)
  self~assertSame(defVal2, s~at)
  self~assertSame(defVal2, s~"[]")
  self~assertEquals(0, s~items)
  s~put("1", "1")
  self~assertEquals(1, s~items)






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
