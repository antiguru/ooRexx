/* An untrapped condition in a started method writes its traceback on that
   activity at once; the program's status is main's. */
m = .k~new~start('BOOM')
m~wait
say m~hasError m~completed
::class k
::method boom
  x = 1/0
