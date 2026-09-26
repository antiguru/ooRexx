/* AddCommandEnvironment through orxfunction's TestAddCommandEnvironment, in
   the FUNCTION group's shapes: a direct handler that always answers -1, a
   redirecting one answering which streams are redirected, and the I/O
   handler's predicates, whose answers are the .true and .false objects. */
call TestAddCommandEnvironment "rc-direct", "direct"
address "rc-direct" ""
say 'direct' rc .rs
signal on syntax name direct
address "rc-direct" "" with input using 123
say 'not here'
direct:
say 'direct with' condition('o')~code condition('o')~message

call TestAddCommandEnvironment "rc-redirecting", "redirecting"
address "rc-redirecting" ""
say 'none' rc .rs
address "rc-redirecting" "" with input using 123
say 'input' rc
address "rc-redirecting" "" with output stem o.
say 'output' rc
address "rc-redirecting" "" with error stem e.
say 'error' rc
address "rc-redirecting" "" with input using 123 output stem o.
say 'input output' rc
address "rc-redirecting" "" with input using 123 error stem e.
say 'input error' rc
address "rc-redirecting" "" with output stem o. error stem e.
say 'output error' rc
address "rc-redirecting" "" with input using 123 output stem o. error stem e.
say 'all' rc
address "rc-redirecting" "" with output stem s. error stem s.
say 'same' rc
address "rc-redirecting" "" with input using 123 output stem s. error stem s.
say 'input same' rc

call TestAddCommandEnvironment "io", "redirecting"
address io 'ISREDIRECTIONREQUESTED' with input using "This is a test"
say 'requested' rc (rc == .true)
address io 'ISREDIRECTIONREQUESTED'
say 'not requested' rc (rc == .false)
address io 'INPUTREDIRECTED' with input using "This is a test"
say 'input redirected' (rc == .true)
address io 'OUTPUTREDIRECTED' with input using "This is a test"
say 'output redirected' (rc == .true)
address io 'ERRORREDIRECTED' with input using "This is a test" error stem e.
say 'error redirected' (rc == .true)
address io 'AREOUTPUTERRORTHESAME' with output stem s. error stem s.
say 'same target' (rc == .true)
address io 'SOMETHINGELSE'
say 'default' (rc == .true) .rs

-- a later registration replaces the earlier one under the same name
call TestAddCommandEnvironment "rc-redirecting", "direct"
address "rc-redirecting" ""
say 'replaced' rc

-- a built-in environment's name is replaced too
call TestAddCommandEnvironment "bash", "direct"
address bash 'exit 3'
say 'bash' rc .rs

-- the name is case-blind; the environment's own spelling reaches the error
signal on syntax name upper
address 'RC-DIRECT' '' with output stem o.
say 'not here'
upper:
say 'upper' condition('o')~code condition('o')~message
exit

::routine TestAddCommandEnvironment PUBLIC EXTERNAL "LIBRARY orxfunction"
