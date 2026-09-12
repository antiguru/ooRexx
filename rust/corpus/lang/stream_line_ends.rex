/* Where a line ends. Only the CR immediately before an LF disappears; a bare
   CR is data; a final line with no terminator is still a line; a NUL is data.
   Every file here is one this program wrote itself. */
say 'line ends'
call build 'crlf.txt', 'a' || '0d0a'x || 'bb' || '0d0a'x
call build 'bare.txt', 'a' || '0d'x || 'bb' || '0d'x
call build 'plain.txt', 'a' || '0a'x || 'bb' || '0a'x
call build 'tail.txt', 'a' || '0a'x || 'bb'
call build 'empty.txt', ''
call build 'nul.txt', 'a' || '00'x || 'bb' || '0a0a'x || 'c'
call build 'blank.txt', '0a0a'x
call show 'crlf.txt'
call show 'bare.txt'
call show 'plain.txt'
call show 'tail.txt'
call show 'empty.txt'
call show 'nul.txt'
call show 'blank.txt'
exit 0

build: procedure
  use arg name, bytes
  s = .Stream~new(name)
  say 'build' name '[' || s~open('write replace') || ']'
  if bytes \== '' then
    say '  charout' s~charout(bytes)
  say '  close [' || s~close || ']'
  return

show: procedure
  use arg name
  s = .Stream~new(name)
  say name 'lines' s~lines 'chars' s~chars
  do 4
    line = s~linein
    if s~state == 'NOTREADY' then leave
    say '  [' || c2x(line) || ']'
  end
  say '  state [' || s~state || '] [' || s~description || ']'
  say '  close [' || s~close || ']'
  return
