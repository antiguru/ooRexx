do f over .context~stackFrames; say '['f~traceLine']' f~line; end
call sub
exit
sub: do f over .context~stackFrames; say '['f~traceLine']'; end
return
