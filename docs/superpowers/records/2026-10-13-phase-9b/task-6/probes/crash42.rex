call charout 'zero.cat', 'de080496'x || '00000000'x || '01000000'x || '00'x
call stream 'zero.cat', 'c', 'close'
say SysGetMessage(1, './zero.cat')
