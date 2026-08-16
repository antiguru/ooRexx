/* A send to a class object resolves against that class's class behaviour:
   its own class methods, and Object's instance methods through the metaclass
   merge. A name neither side answers is 97.1 naming the class object. */
say .K~hasMethod('M')
say .K~hasMethod('LENGTH')
say .K~isNil
say .K~nosuchmsg

::class K

::method m class
  return 1
