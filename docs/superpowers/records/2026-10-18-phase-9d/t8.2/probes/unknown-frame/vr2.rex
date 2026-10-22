a = .directory~new; a~setmethod("go","return 1/0")
r = >a
say r~go
