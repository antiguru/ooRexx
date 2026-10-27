/* Class~define while instances made under each definition stay live: every
   define retires the class's instance behaviour, and every collection that
   frees objects looks for the behaviours no live instance holds. */
n = 3000
keep = .array~new(50)
do i = 1 to n
  .k~define('M', 'return' i)
  keep[i // 50 + 1] = .k~new
  do j = 1 to 20
    x = .array~of(i, j, .directory~new)
  end
end
say keep[1]~m keep[50]~m
::class k
