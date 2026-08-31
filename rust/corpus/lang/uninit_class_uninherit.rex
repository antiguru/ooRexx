/* ~uninherit takes the class-side UNINIT back off QQ.  The registration is
   never undone -- RexxClass::checkUninit only sets -- so QQ is still reached
   by the termination sweep and runs nothing, because RexxObject::uninit
   re-tests hasMethod(UNINIT) at delivery. */
say 'main'
.QQ~inherit(.MX)
say 'inherited'
.QQ~uninherit(.MX)
say 'uninherited'
exit 0

::class qq

::class mx mixinclass Object
::method uninit class
  say 'u' self~id
