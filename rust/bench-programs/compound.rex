/* Stem/compound-variable dimension: the -24% memo prototype's workload.
   500 tails is inside the prototype's measured 100-10000 sweet spot. */
n = 5000000
tails = 500
t. = 0
do i = 1 to n
    k = i // tails
    t.k = t.k + 1
end
total = 0
do k = 0 to tails - 1
    total = total + t.k
end
say total
