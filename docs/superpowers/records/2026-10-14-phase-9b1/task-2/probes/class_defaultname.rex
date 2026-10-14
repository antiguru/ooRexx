say .plain length(.plain)
.plain~addit
say .plain length(.plain) .plain~string
say .withname length(.withname)
exit
::class plain
::method addit class; self~setMethod('STRING', 'return "SM"')
::class withname
::method defaultname class; return 'DN'
