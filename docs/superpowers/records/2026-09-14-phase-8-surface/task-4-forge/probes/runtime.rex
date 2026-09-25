say 'before'
call loadlib
say 'after' usestash()
exit
loadlib: 
  say .context~package~loadLibrary('forgea')
  return
