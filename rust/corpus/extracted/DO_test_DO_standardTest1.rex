/* extracted from DO::test_DO_standardTest1 */
::routine main public
   cnt.=0
   i=0;b=0;t=0;
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 4)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 4)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 5)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 5)
   is=0;/**/Do/**/ k/**/=/**/ 6 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 5)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 5)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 4)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 4)
   is=0;/**/Do/**/ k/**/=/**/ 3 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 4)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 4)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 4)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 4)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ 0 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 4)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 4)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 5)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 5)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   end
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ -3 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 5)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 5)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 4)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 4)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 3)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 1)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 10;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 5;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ 0;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 14)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 6)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ -5;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 15-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 7-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ 0 /**/to/**/ -10;is=is+1;
   If is= 0-1 Then Leave
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ -3 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 10;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ 0;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ -5;is=is+1;
   End
   self~AssertSame(is, 0)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 15/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 7/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 2)
   is=0;/**/Do/**/ k/**/=/**/ -6 /**/for/**/ 0/**/by/**/ -4 /**/to/**/ -10;is=is+1;
   End
   self~AssertSame(is, 0)

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
