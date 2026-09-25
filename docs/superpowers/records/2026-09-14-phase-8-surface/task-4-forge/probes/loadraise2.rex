say 'main'
signal on syntax name trapped
call load
say 'not reached'
exit
trapped:
say 'trapped' condition('o')~code
say 'x' usestash()
say 'again' .context~package~loadLibrary('forgeload')
exit
load:
  say .context~package~loadLibrary('forgeload')
  return
