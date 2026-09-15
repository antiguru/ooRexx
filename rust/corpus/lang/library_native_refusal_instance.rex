/* A native method's refusal of its argument is reported against the package
   its EXTERNAL directive was written in, with no line: a RexxClassObject that is not a class is 88.914. */
say 'before'
t = .T~new
say t~classarg(1)
say 'after'

::requires 'pk.cls'
