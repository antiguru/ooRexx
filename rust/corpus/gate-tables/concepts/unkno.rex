/* provide.xml `unkno`: when the object has no matching method the language
   processor calls UNKNOWN, passing the name of the method that was not
   located and an array holding the arguments of the original message. */
say .k~zork(1, 2)

::class k
::method unknown class
  use arg name, arguments
  return 'unknown:' name 'with' arguments~items 'argument(s):' arguments[1] arguments[2]
