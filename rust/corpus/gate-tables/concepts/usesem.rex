/* provide.xml `usesem`: setMethod adds a method directly to one instance and
   the Class class's enhanced method creates an instance carrying extra
   methods; the methods and object variables so defined form a scope of their
   own, so neither reaches the class's other instances. setMethod is a private
   message, so it is sent to self from inside a method. */
o = .k~new
o~addExtra
say 'setmethod' o~extra
say 'not-shared' .k~new~hasMethod("EXTRA")
t = .stringtable~new
t["ENHANCED"] = "return 'enhanced'"
e = .k~enhanced(t)
say 'enhanced' e~enhanced
say 'still-not-shared' .k~new~hasMethod("ENHANCED")

::class k
::method addExtra
  self~setMethod("EXTRA", "return 'one-off'")
