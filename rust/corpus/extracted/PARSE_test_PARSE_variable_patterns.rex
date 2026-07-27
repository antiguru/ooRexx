/* extracted from PARSE::test_PARSE_variable_patterns */
::routine main public

   /* Using a variable as a string pattern                          */
   /*  The variable (delim) is set in the same template             */
   -- SAY "Enter a date (mm/dd/yy format). =====> " /* assume 11/15/98 */
   PUSH "11/15/98" -- rgf: push assumed date
   pull date
   parse var date month 3 delim +1 day +2 (delim) year
   /* Sets: month='11'; delim='/'; day='15'; year='98'  */
   self~assertSame('11', month)
   self~assertSame('/', delim)
   self~assertSame('15', day)
   self~assertSame('98', year)


   /* Using a variable as a positional pattern                      */
   dataline = '12 26 .....Samuel ClemensMark Twain'
   parse var dataline pos1 pos2 6 =(pos1) realname =(pos2) pseudonym
   /* Assigns: realname='Samuel Clemens'; pseudonym='Mark Twain'    */
   self~assertSame('Samuel Clemens', realname)
   self~assertSame('Mark Twain', pseudonym)


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
