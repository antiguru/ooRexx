/* provide.xml `reqstr`: to obtain a string value the language processor sends
   request("STRING"); an object with a makeString method answers it, and one
   without gets the NOSTRING condition when it is trapped and a `string`
   message when it is not. */
say .k
say 'request-with' .k~request("STRING")
say 'request-without' .p~request("STRING")
say 'string' .p~string
say 'untrapped' .p
signal on nostring name noStr
say 'trapped' .p
say 'not reached'
exit 0

noStr:
say 'nostring raised'

::class k
::method makeString class
  return "K says hello"

::class p
