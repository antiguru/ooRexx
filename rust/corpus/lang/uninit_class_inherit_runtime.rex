/* ~inherit gives a class a class-side UNINIT after two other classes are
   already registered, so QQ's entry is made third and lands second in the
   bucket it shares with ZED. */
say 'main'
.QQ~inherit(.MX)
say 'inherited'
exit 0

::class qq

::class zed
::method uninit class
  say 'u' self~id

::class mx mixinclass Object
::method uninit class
  say 'u' self~id
