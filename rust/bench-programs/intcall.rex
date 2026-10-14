/* Internal package routines called n times each: FILESPEC of the REXX package and
   SysFileExists of rexxutil, which run no activation. */
n = 150000
t = 0
do i = 1 to n
    t = t + length(filespec('N', '/usr/lib/liba.so')) + SysFileExists('/nonexistent/liba.so')
end
say t
