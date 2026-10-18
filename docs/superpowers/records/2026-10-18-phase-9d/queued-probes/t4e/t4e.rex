d = .directory~new
d~setMethod('go', 'return 1/0')
say d~go
