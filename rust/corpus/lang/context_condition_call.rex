/* .context~condition inside a CALL ON handler. */
call on error
'exit 3'
say 'after' .context~condition
exit
error:
c = .context~condition
say c~class~id c~condition c~rc
return
