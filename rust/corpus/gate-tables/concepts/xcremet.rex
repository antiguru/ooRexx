/* provide.xml `xcremet`: a ::CLASS directive with a quoted string identifier
   creates a subclass of Object, and the string is both the class name and the
   environment symbol that locates it. ::METHOD directives following it add
   methods to that class. */
say 'id' .Account~id
say 'superclass' .Account~superClass~id
say 'type' .Account~type

::class "Account"
::method type class
  return "an account"
