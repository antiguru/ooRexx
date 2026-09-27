/* orxfunction's handlers under every ::OPTIONS ... SYNTAX a handler's
   condition can meet. They raise nothing, so nothing escalates: a direct
   handler's -1 and a redirecting one's answer are RC, and .RS is 0. The
   escalations of a raised condition are in the Task 6 forge's opts_*.rex. */
call TestAddCommandEnvironment "rc-direct", "direct"
address "rc-direct" ""
say 'direct' rc .rs

call TestAddCommandEnvironment "rc-redirecting", "redirecting"
address "rc-redirecting" "" with output stem o.
say 'redirecting' rc .rs

call TestAddCommandEnvironment "io", "redirecting"
address io 'INPUTOUTPUT' with input using (('a', 'b')) output stem o.
say 'io' rc .rs o.0 o.1 o.2
address io 'INPUTERROR' with input using "e" error stem e.
say 'io error' rc .rs e.0 e.1

-- a command's own ERROR still escalates
signal on syntax name command
address system 'exit 3'
say 'not here'
command:
say 'command' condition('o')~code condition('o')~message
exit

::routine TestAddCommandEnvironment PUBLIC EXTERNAL "LIBRARY orxfunction"
::options error syntax failure syntax notready syntax lostdigits syntax nostring syntax
