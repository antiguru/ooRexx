/* NOTREADY under both ERROR SYNTAX and NOTREADY SYNTAX */
call AddCmd 'xd', 'd'
signal on syntax name l1
address xd 'RAISE NOTREADY'
say 'after notready' rc .rs
l1: c = condition('o'); say 'notready:' c~code c~message rc .rs
exit
::requires 'cmd' LIBRARY
::options error syntax notready syntax
