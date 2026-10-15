a = .array~new
do i = 1 to 200000; a[i] = .object~new; end
do j = 1 to 20000; z = copies('x', 10000); end
say a~items
