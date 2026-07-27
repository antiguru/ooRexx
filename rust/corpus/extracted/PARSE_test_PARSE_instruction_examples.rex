/* extracted from PARSE::test_PARSE_instruction_examples */
::routine main public

    -- test ARG and PARSE ARG
    call arg_sub "aHa" olala, "AHA", "OLALA", 1, "aHa"

    -- LINEIN: needs to read from STDIN
    call do_test_linein
    tmpRC = result

    self~assertSame(17, tmpRC)

    -- PULL
    push "80 4"
    parse pull one two
    self~assertSame("80", one)
    self~assertSame("4", two)

    push "anton"
    parse pull var1
    self~assertSame("anton", var1)

    push "anton"
    pull var1
    self~assertSame("ANTON", var1)


    -- SOURCE
    PARSE SOURCE opSys invocation path
    self~assertEquals("METHOD", invocation)

    -- VALUE
    PARSE VALUE WITH a b c
    self~assertSame("", a)
    self~assertSame("", b)
    self~assertSame("", c)

    PARSE VALUE 1+20 WITH a +1 b
    self~assertSame("2", a)
    self~assertSame("1", b)

    -- VERSION
    PARSE VERSION language +4 .
    self~assertSame("REXX", language)



   exit -- exit testcase


 arg_sub:
      /* ARG with source string named in Rexx program invocation       */
      /*  Program name is PALETTE.  Specify 2 primary colors (yellow,  */
      /*   red, blue) on call.   Assume call is: palette red blue      */
      arg var1 var2, var1a, var2a, run          /* Assigns: var1='RED'; var2='BLUE' */
      self~assertSame(var1a, var1)
      run=run+1
      self~assertSame(var2a, var2)

      run=run+1
      -- do not uppercase
      parse arg var1 ., ., ., ., var1a with
      self~assertSame(var1a, var1)
      return


do_test_linein: procedure             -- create external Rexx program for testing

   filename="tmpTestPARSElinein"
   fileInput=filename||"_input.txt"
   filename=filename||".rex"

      ------------------------------
     -- create input file
   s=.stream~new(fileInput)~~open("replace")   -- create empty file
   s~~lineout("a=8 c=9")~close

      ------------------------------
     -- create rexx-file containing PARSE LINEIN, will set RC=17 if successful
   s=.stream~new(fileName)~~open("replace")   -- create empty file
   s~~lineout("/*" date("S") time() ", ---rgf */")~~lineout("")

   s~~lineout("parse linein 'a=' num1 'c=' num2     /* Assume: 8 and 9          */")
   s~~lineout("sum=num1+num2                        /* Enter: a=8 b=9 as input  */")
   s~~lineout("exit sum   -- set RC to '17'") ~~lineout("")
   s~close

      ------------------------------
     -- invoke program which does the PARSE LINEIN and returns the sum of the parsed values
   parse upper source opsys +1

   currentEnvironment=address()              -- save current environment
   address (ooRexxUnit.getShellName())       -- set environment to shell

   -- address cmd "rexx" filename "<"fileinput  -- run as external command redirect input
   "rexx" filename "<"fileinput  -- run as external command redirect input
   tmpRC=rc       -- should be 17 (8+9)

   address (currentEnvironment)              -- restore current environment

      ------------------------------
     -- delete the files just created (clean up)
   call sysFileDelete fileName  -- delete file
   call sysFileDelete fileInput -- delete file

   return tmpRC


  -- ---> ---> END: from "rexxref" (v.3.0.0 rev2 2005-05-31, chapter 10 Parsing: END <--- <---


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
