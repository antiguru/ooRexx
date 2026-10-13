parse value SysCreatePipe() with reading writing
out = .stream~new('/proc/self/fd/'writing)
say out~open('write append')
say out~lineout('hello')
say out~close
in = .stream~new('/proc/self/fd/'reading)
say in~open('read')
say in~chars in~lines
line = in~linein
say '['line']' in~state in~description
say in~close
name = '/proc/self/fdinfo/'reading
do 10
  l = linein(name)
  if l = '' then leave
  say l
end
