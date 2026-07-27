/* extracted from PARSE::test_PARSE_Positional_Patterns */
::routine main public
         --  1234+6789!1234+6789!1234+6789!1234+6789!
   record.1='Clemens   Samuel    Mark Twain          '
   var.1.1 ='Clemens   '
   var.1.2 ='Samuel    '
   var.1.3 ='Mark Twain          '

   record.2='Evans     Mary Ann  George Eliot        '
   var.2.1 ='Evans     '
   var.2.2 ='Mary Ann  '
   var.2.3 ='George Eliot        '

   record.3='Munro     H.H.      Saki                '
   var.3.1 ='Munro     '
   var.3.2 ='H.H.      '
   var.3.3 ='Saki                '

   /* Parsing with absolute positional patterns in template         */
   do n=1 to 3
     parse var record.n lastname 11 firstname 21 pseudonym
     self~assertSame(var.n.1, lastname)
     self~assertSame(var.n.2, firstname)
     self~assertSame(var.n.3, pseudonym)
   end


   /* Parsing with absolute positional patterns in template         */
   do n=1 to 3
     parse var record.n lastname +10 firstname + 10 pseudonym
     self~assertSame(var.n.1, lastname)
     self~assertSame(var.n.2, firstname)
     self~assertSame(var.n.3, pseudonym)
   end


   /* Backing up to an earlier position (with absolute positional)  */
   string='astronomers'
   parse var string 2 var1 4 1 var2 2 4 var3 5 11 var4
   res1=string 'study' var1||var2||var3||var4
   self~assertSame(res1, "astronomers study stars")


   /* Backing up to an earlier position (with relative positional)  */
   string='astronomers'
   parse var string 2 var1 +2 -3 var2 +1 +2 var3 +1 +6 var4
   res2=string 'study' var1||var2||var3||var4
   self~assertSame(res2, "astronomers study stars")


   /* Making several assignments                                    */
   books='Silas Marner, Felix Holt, Daniel Deronda, Middlemarch'
   parse var books 1 Eliot 1 Evans
   /* Assigns the (entire) value of books to Eliot and to Evans.    */
   self~assertSame(Eliot, Evans)



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
