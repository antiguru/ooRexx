do n over '/a/b//../c', '/a/b/../c', '/a//b', '/a/b//./c', '/a/b///../c', '/a/b/.//../c', '/a/b//..', '//..', '/a/./../b'
  say n '->' qualify(n)
end
