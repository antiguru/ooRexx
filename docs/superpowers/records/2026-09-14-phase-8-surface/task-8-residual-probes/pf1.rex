signal on syntax name s6
interpret 'x = 1 + "y"'
s6:
say '['.context~stackframes[1]~name']'
signal on syntax name s7
x = 1 + 'q'
s7: c = condition('O'); do f over c~stackframes; say f~type '['f~name']' f~line; end
say .k~new~frames
exit
::class k
::method frames
  s = ''
  do f over .context~stackframes; s = s f~type'['f~name']'; end
  return s
