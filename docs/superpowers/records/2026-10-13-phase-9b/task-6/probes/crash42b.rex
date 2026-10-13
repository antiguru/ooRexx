call charout 'far.cat', 'de080496'x || 'a0860100'x || '01000000'x || copies('00'x, 20)
call stream 'far.cat', 'c', 'close'
say SysGetMessage(1, './far.cat')
