/* extracted from LabelOption::test_SELECT_LABEL_error */
::routine main public

   -- This test should produce a syntax error, test just that.
   str="i=0                       ;" -
   "SELECT label aha1             ;" -
   "   when i=-1 then  nop        ;" -
   "   when i=0 then              ;" -
   "            do                ;" -
   "               i=i+1          ;" -
   "               do label aha2  ;" -
   "                  leave aha2  ;" -
   "                  i = i + 10  ;" -
   "               end            ;" -
   "               i=i+1          ;" -
   "               leave          ;" -
   "            end               ;" -
   "   otherwise nop              ;" -
   "END nixi                      ;" -

   self~expectSyntax( 10.4 )
   interpret str


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
