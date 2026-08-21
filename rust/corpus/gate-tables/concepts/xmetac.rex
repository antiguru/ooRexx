/* provide.xml `xmetac`: the Class class is the metaclass of every class Rexx
   provides, so instances of .Class are themselves classes; METACLASS names a
   different class for a class object to be an instance of. */
say 'provided' .Array~id .Array~class~id
say 'default' .p~id .p~class~id
say 'metaclass' .k~id .k~class~id

::class mc subclass class
::class k metaclass mc
::class p
