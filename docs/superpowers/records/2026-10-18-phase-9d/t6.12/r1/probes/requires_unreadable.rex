signal on syntax
call loadit
exit
loadit: .context~package~loadPackage('noperm.cls'); say "ok"; return
syntax: say "syntax" condition('o')~code condition('o')~message
