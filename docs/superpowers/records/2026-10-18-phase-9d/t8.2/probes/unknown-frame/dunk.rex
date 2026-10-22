d = .directory~new
d~setMethod("unknown", "return 1/0")
say d~go
