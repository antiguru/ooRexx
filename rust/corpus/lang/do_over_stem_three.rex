/* DO OVER a stem binds its tails; sorted, because the order is licensed
   (deviation 1). */
s.1 = 'x'; s.2 = 'y'; s.abc = 'z'
l = .array~new
do t over s.; l~append(t); end
say l~sort~makestring('L', ' ')
