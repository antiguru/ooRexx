/* Body-byte churn over a live set: 20,000 objects held while n strings of
   10,000 bytes are made and dropped, so every collection marks the live set
   and how often one runs follows the byte trigger. */
n = 100000
a = .array~new
do i = 1 to 20000
    a[i] = .object~new
end
do j = 1 to n
    z = copies('x', 10000)
end
say a~items
