l = .list~new; do i = 1 to 100000; l~append(i); end
call time 'r'
do i = 1 to 2000; l~remove(l~first); end
t1 = time('r')
do i = 1 to 2000; l~remove(l~last); end
t2 = time('r')
say l~items (t1 < 0.5) (t2 < 0.5)
