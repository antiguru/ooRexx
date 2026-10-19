.traceOutput~destination(.refusing~new)
t = .k~new
t~one
call syssleep 0.1
say 'after'
::requires 'rpd.rex'
::class k
::method one
  forward message "onetwo"
::method onetwo
  reply
::options trace labels
