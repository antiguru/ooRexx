.k~new~start("spin"); m = .k~new~start("spin2"); call syssleep 0.2; say "ok" m~halt("stop"); m~wait; say "ok" m~hasError m~errorCondition~description
::class k
::method spin
::method spin2
  do forever; nop; end
