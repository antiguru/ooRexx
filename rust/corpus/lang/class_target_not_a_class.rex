/* A ::CLASS directive whose SUBCLASS target names an object that is not a
   class: 99.949 at translation, where a name nothing answers is 98.909. */
say .k~new~go
::class k subclass rexxinfo
::method go
  expose x; x = 3; return x
