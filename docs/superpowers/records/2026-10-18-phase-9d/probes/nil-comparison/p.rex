signal on nostring
test = .nil~string
say (test > .nil) (test < .nil) (test >= .nil) (test <= .nil) (test >> .nil)
test = 120
say (test > .nil)
exit
nostring: say 'NOSTRING at' sigl condition('d')
