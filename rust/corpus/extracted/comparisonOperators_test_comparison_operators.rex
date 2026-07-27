/* extracted from comparisonOperators::test_comparison_operators */
::routine main public
   -- none of these should raise nostring conditions
   signal on nostring

   self~assertTrue(5=5) /* equal */
   self~assertTrue(42\=41) /* All of these are */
   self~assertTrue(42><41) /* "not equal" */
   self~assertTrue(42<>41)
   self~assertTrue(13>12) /* Variations of */
   self~assertTrue(12<13) /* less than and */
   self~assertTrue(13>=12) /* greater than */
   self~assertFalse(12\<13)
   self~assertTrue(12<=13)
   self~assertTrue(12\>13)

   self~assertTrue(5~'='(5)) /* equal */
   self~assertTrue(42~'\='(41)) /* All of these are */
   self~assertTrue(42~'><'(41)) /* "not equal" */
   self~assertTrue(42~'<>'(41))
   self~assertTrue(13~'>'(12)) /* Variations of */
   self~assertTrue(12~'<'(13)) /* less than and */
   self~assertTrue(13~'>='(12)) /* greater than */
   self~assertFalse(12~'\<'(13))
   self~assertTrue(12~'<='(13))
   self~assertTrue(12~'\>'(13))


   -- testcases added for [bugs:#1358] Strict comparison gives unexpected result
   -- numeric standard integer/integer
   self~assertFalse(2=12)
   self~assertTrue(2\=12)
   self~assertTrue(2<>12)
   self~assertTrue(2><12)
   self~assertFalse(2>12)
   self~assertFalse(2>=12)
   self~assertFalse(2\<12)
   self~assertTrue(2<12)
   self~assertTrue(2<=12)
   self~assertTrue(2\>12)

   -- numeric strict integer/integer
   self~assertFalse(2==12)
   self~assertTrue(2\==12)
   self~assertTrue(2>>12)
   self~assertTrue(2>>=12)
   self~assertTrue(2\<<12)
   self~assertFalse(2<<12)
   self~assertFalse(2<<=12)
   self~assertFalse(2\>>12)

   -- numeric standard integer/string
   self~assertFalse(2="12")
   self~assertTrue(2\="12")
   self~assertTrue(2<>"12")
   self~assertTrue(2><"12")
   self~assertFalse(2>"12")
   self~assertFalse(2>="12")
   self~assertFalse(2\<"12")
   self~assertTrue(2<"12")
   self~assertTrue(2<="12")
   self~assertTrue(2\>"12")

   -- numeric strict integer/string
   self~assertFalse(2=="12")
   self~assertTrue(2\=="12")
   self~assertTrue(2>>"12")
   self~assertTrue(2>>="12")
   self~assertTrue(2\<<"12")
   self~assertFalse(2<<"12")
   self~assertFalse(2<<="12")
   self~assertFalse(2\>>"12")

   -- numeric standard string/integer
   self~assertFalse("2"=12)
   self~assertTrue("2"\=12)
   self~assertTrue("2"<>12)
   self~assertTrue("2"><12)
   self~assertFalse("2">12)
   self~assertFalse("2">=12)
   self~assertFalse("2"\<12)
   self~assertTrue("2"<12)
   self~assertTrue("2"<=12)
   self~assertTrue("2"\>12)

   -- numeric strict string/integer
   self~assertFalse("2"==12)
   self~assertTrue("2"\==12)
   self~assertTrue("2">>12)
   self~assertTrue("2">>=12)
   self~assertTrue("2"\<<12)
   self~assertFalse("2"<<12)
   self~assertFalse("2"<<=12)
   self~assertFalse("2"\>>12)

   -- numeric standard string/string
   self~assertFalse("2"="12")
   self~assertTrue("2"\="12")
   self~assertTrue("2"<>"12")
   self~assertTrue("2"><"12")
   self~assertFalse("2">"12")
   self~assertFalse("2">="12")
   self~assertFalse("2"\<"12")
   self~assertTrue("2"<"12")
   self~assertTrue("2"<="12")
   self~assertTrue("2"\>"12")

   -- numeric strict integer/integer
   self~assertFalse("2"=="12")
   self~assertTrue("2"\=="12")
   self~assertTrue("2">>"12")
   self~assertTrue("2">>="12")
   self~assertTrue("2"\<<"12")
   self~assertFalse("2"<<"12")
   self~assertFalse("2"<<="12")
   self~assertFalse("2"\>>"12")
   -- END testcases added for [bugs:#1358]


   -- number string comparisons
   self~assertTrue(5.5=5.5) /* equal */
   self~assertTrue(42.1\=41.1) /* All of these are */
   self~assertTrue(42.1><41.1) /* "not equal" */
   self~assertTrue(42.1<>41.1)
   self~assertTrue(13.1>12.1) /* Variations of */
   self~assertTrue(12.1<13.1) /* less than and */
   self~assertTrue(13.1>=12.1) /* greater than */
   self~assertFalse(12.1\<13.1)
   self~assertTrue(12.1<=13.1)
   self~assertTrue(12.1\>13.1)

   self~assertTrue(5.5~'='(5.5)) /* equal */
   self~assertTrue(42.1~'\='(41.1)) /* All of these are */
   self~assertTrue(42.1~'><'(41.1)) /* "not equal" */
   self~assertTrue(42.1~'<>'(41.1))
   self~assertTrue(13.1~'>'(12.1)) /* Variations of */
   self~assertTrue(12.1~'<'(13.1)) /* less than and */
   self~assertTrue(13.1~'>='(12.1)) /* greater than */
   self~assertFalse(12.1~'\<'(13.1))
   self~assertTrue(12.1~'<='(13.1))
   self~assertTrue(12.1~'\>'(13.1))

   -- repeat all of these with the left-hand side being explicitly
   -- a string value.  Because there are special hidden integer and numberstring
   -- classes used internally, this can make an actual different

   self~assertTrue('5'=5) /* equal */
   self~assertTrue('42'\=41) /* All of these are */
   self~assertTrue('42'><41) /* "not equal" */
   self~assertTrue('42'<>41)
   self~assertTrue('13'>12) /* Variations of */
   self~assertTrue('12'<13) /* less than and */
   self~assertTrue('13'>=12) /* greater than */
   self~assertFalse('12'\<13)
   self~assertTrue('12'<=13)
   self~assertTrue('12'\>13)

   self~assertTrue('5'~'='(5)) /* equal */
   self~assertTrue('42'~'\='(41)) /* All of these are */
   self~assertTrue('42'~'><'(41)) /* "not equal" */
   self~assertTrue('42'~'<>'(41))
   self~assertTrue('13'~'>'(12)) /* Variations of */
   self~assertTrue('12'~'<'(13)) /* less than and */
   self~assertTrue('13'~'>='(12)) /* greater than */
   self~assertFalse('12'~'\<'(13))
   self~assertTrue('12'~'<='(13))
   self~assertTrue('12'~'\>'(13))

   -- number string comparisons
   self~assertTrue('5.5'=5.5) /* equal */
   self~assertTrue('42.1'\=41.1) /* All of these are */
   self~assertTrue('42.1'><41.1) /* "not equal" */
   self~assertTrue('42.1'<>41.1)
   self~assertTrue('13.1'>12.1) /* Variations of */
   self~assertTrue('12.1'<13.1) /* less than and */
   self~assertTrue('13.1'>=12.1) /* greater than */
   self~assertFalse('12.1'\<13.1)
   self~assertTrue('12.1'<=13.1)
   self~assertTrue('12.1'\>13.1)

   self~assertTrue('5.5'~'='(5.5)) /* equal */
   self~assertTrue('42.1'~'\='(41.1)) /* All of these are */
   self~assertTrue('42.1'~'><'(41.1)) /* "not equal" */
   self~assertTrue('42.1'~'<>'(41.1))
   self~assertTrue('13.1'~'>'(12.1)) /* Variations of */
   self~assertTrue('12.1'~'<'(13.1)) /* less than and */
   self~assertTrue('13.1'~'>='(12.1)) /* greater than */
   self~assertFalse('12.1'~'\<'(13.1))
   self~assertTrue('12.1'~'<='(13.1))
   self~assertTrue('12.1'~'\>'(13.1))

   -- identical
   self~assertTrue('space'  ==  'space')  /* Strictly equal */
   self~assertTrue('space'  \== ' space')  /* Strictly not equal */
   self~assertTrue('space'  >>  ' space')  /* Variations of */
   self~assertTrue(' space' <<  'space')  /* strictly greater */
   self~assertTrue('space'  >>= ' space')  /* than and less than */
   self~assertTrue('space'  \<< ' space')
   self~assertTrue(' space' <<= 'space')
   self~assertTrue(' space' \>> 'space')

   self~assertTrue('space'~'=='('space'))  /* Strictly equal */
   self~assertTrue('space'~'\=='(' space'))  /* Strictly not equal */
   self~assertTrue('space'~'>>'(' space'))  /* Variations of */
   self~assertTrue(' space'~'<<'('space'))  /* strictly greater */
   self~assertTrue('space'~'>>='(' space'))  /* than and less than */
   self~assertTrue('space'~'\<<'(' space'))
   self~assertTrue(' space'~'<<='('space'))
   self~assertTrue(' space'~'\>>'('space'))

   return
   nostring:
   self~assertTrue(.false, "Unexpected NOSTRING condition raised at" sigl)

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
