trace r
interpret 'say sourceline(); say .context~package~sourceSize; call lab; say "back"'
interpret 'interpret "say 1+2"'
signal on syntax
interpret 'x = 1 + "z"'
exit
syntax: do l over condition('o')~traceback; say l; end
say condition('o')~position
exit
lab: say 'lab'
return
