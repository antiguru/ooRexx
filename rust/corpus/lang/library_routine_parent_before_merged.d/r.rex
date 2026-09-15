say 'library' rxcalcsqrt(16)
say 'public' pubname(16)
say 'findRoutine' .context~package~findRoutine('MAINR')
say 'findRoutine library' .context~package~findRoutine('RXCALCSQRT')~call(16)
say 'findRoutine public' .context~package~findRoutine('PUBNAME')~call(16)
say 'merged' RxCalcPower(2, 3)
::requires 'rxmath' LIBRARY
::requires 'pub.cls'
