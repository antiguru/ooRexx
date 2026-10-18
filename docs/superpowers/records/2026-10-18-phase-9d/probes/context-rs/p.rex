call m
exit
m: procedure
say (.context~rs == .nil)
address "" "echo foo" with output using (.Array~new)
say .rs .context~rs
"true"
say .rs .context~rs
::routine r
