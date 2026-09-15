/* A native method's refusal of its argument is reported against the package
   its EXTERNAL directive was written in, with no line: a negative nonnegative_wholenumber_t is 88.904. */
say 'before'
t = .T~new
say t~nonneg(-1)
say 'after'

::requires 'pk.cls'
