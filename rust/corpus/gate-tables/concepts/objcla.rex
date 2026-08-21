/* provide.xml `objcla`: an object class is a factory for objects, and an
   object acquires the instance methods its class has AT THE TIME OF ITS
   CREATION -- a class that gains a method later does not give it to an
   object that already exists. */
o = .k~new
say 'before' o~hasMethod("EXTRA")
.k~define("EXTRA", "return 'extra'")
say 'after' o~hasMethod("EXTRA")
say 'fresh' .k~new~hasMethod("EXTRA")

::class k
