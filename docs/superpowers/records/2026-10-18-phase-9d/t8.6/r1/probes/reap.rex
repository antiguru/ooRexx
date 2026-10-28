a = .w~new~start('RUN')
do 400 until SysFileExists('started'); call SysSleep 0.05; end
say 'wait' SysWait()
say 'cmd' a~result
::class w
::method run
call on failure name failed
'touch started; sleep 0.6; exit 5'
return 'rc' rc
failed:
say 'failure' rc
return
