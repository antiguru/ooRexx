arr = .array~new
.traceObject~collector = arr
.traceObject~option = 'P'
code = .routine~new("tv", ("o = 1", "d = o + 1", "return d", "::options trace r"))
say code~call
.traceObject~collector = .nil
say arr~items
do t over arr; say t~hasEntry("VARIABLE") t~traceLine; end
