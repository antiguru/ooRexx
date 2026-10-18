t=.test~new
t~one
call syssleep 0.001
res=t~three
call syssleep 0.001
say res
::class test
::method one
  forward message "onetwo"
  i=i/0
::method onetwo
  reply
::method three
  reply 42
  forward message "threefour" continue
  if result<>42 then i=i/0
::method threefour
  return 42
::options trace labels
