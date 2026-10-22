d = .directory~new
say d~unknown("GO", .array~new)
d~setMethod("go", "return 1/0")
say d~unknown("GO", .array~new)
