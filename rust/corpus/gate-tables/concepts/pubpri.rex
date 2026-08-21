/* provide.xml `pubpri`: a method is public, package-scope or private. Any
   object can send a message that runs a public method; a private method is
   reachable only from within the same object. */
say 'public' .k~pub
say 'private-from-inside' .k~callPriv
say 'private-from-outside' .k~priv

::class k
::method pub class
  return 'public'
::method priv class private
  return 'private'
::method callPriv class
  return self~priv
