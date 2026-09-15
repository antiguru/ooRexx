/* Package~loadLibrary inside a required package's routine makes the library's
   routines callable by name from every package: the one that loaded it, one
   translated before the load, this program, and one built after it. No
   package's own lookup holds them, so findRoutine answers .nil. */
say 'before' callsq()
say 'load' loadit()
say 'after' callsq()
say 'main' RxCalcSqrt(64)
say 'findRoutine' .context~package~findRoutine('RxCalcSqrt')
say 'late' latecall()
::requires 'caller.cls'
::requires 'loader.cls'
