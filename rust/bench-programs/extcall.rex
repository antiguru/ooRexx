/* liborxfunction's TestIntArg called n times; needs its directory on LD_LIBRARY_PATH. */
n = 3000000
t = 0
do i = 1 to n
    t = TestIntArg(i)
end
say t

::requires 'orxfunction' LIBRARY
