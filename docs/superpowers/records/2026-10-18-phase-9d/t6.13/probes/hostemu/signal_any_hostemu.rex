signal on any
address "hostemu" "HI"
say 'after' rc
exit
any: say condition('C') condition('D') rc
exit
::requires "hostemu" LIBRARY
