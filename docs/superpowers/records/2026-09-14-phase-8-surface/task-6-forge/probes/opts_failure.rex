/* USER and NOTREADY under FAILURE SYNTAX alone */
call AddCmd 'xd', 'd'
signal on syntax name l1
address xd 'RAISE USER FOO'
say 'after user' rc .rs
address xd 'RAISE NOTREADY'
say 'after notready' rc .rs
exit
l1: c = condition('o'); say 'syntax:' c~code c~message
::requires 'cmd' LIBRARY
::options failure syntax
