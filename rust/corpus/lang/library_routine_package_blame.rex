/* A required package's ::ROUTINE binds the entry's shared code, so a call of
   the entry by its own name is reported against that package with no line. */
say 'start'
say RxCalcSqrt(1, 2, 3)
::requires 'pk.cls'
