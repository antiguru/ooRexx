/* Hash-collection construction: a fresh Directory, StringTable and Table
   every pass, none kept, 50,000 passes. The path NEW and INIT take to give
   each its store. */
n = 50000
total = 0
do i = 1 to n
    d = .directory~new
    s = .stringtable~new
    t = .table~new
    total = total + d~items + s~items + t~items
end
say total n
