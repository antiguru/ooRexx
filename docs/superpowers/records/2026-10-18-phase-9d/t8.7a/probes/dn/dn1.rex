say .withname
say .withname~string
say .withname~objectName
say .withname~defaultName
say .withname~id
say .withname~new
.withname~objectName = 'named'
say .withname
say .withname~defaultName
say .plain
say .plain~new~class
say .sub
say .sub~new
say .withname~class
::class withname
::method defaultname class
  return 'DN'
::method defaultname
  return 'idn'
::class plain
::class sub subclass withname
