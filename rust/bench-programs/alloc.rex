/* Allocation-churn dimension: enough short-lived objects per iteration to
   force collections, none of them retained past the iteration that made them. */
n = 3000000
total = 0
do i = 1 to n
    a = .array~of(i, i + 1, i + 2)
    s = .string~new("item")
    total = total + a~size + s~length
end
say total
