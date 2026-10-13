list = '/a/b//../c /a/b///../c /a/b/.//../c /a/b//.. /a/b/c/..//../d /a/b/c//../../d /a/.hidden/../c /a/.config/../x /a/b/..x/../c /a/b/.../../c /a/.b/../../c /a/b/./../c /a//b/../c //a/../b /a/./../b /a/b/../../.. /a/b/../c /a/b/. /a/b/./c'
do i = 1 to words(list)
  n = word(list, i)
  say left(n, 22) qualify(n)
end
