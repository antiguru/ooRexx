/* A namespace package's own dependency, reached transitively through it. */
say 'nsdep prologue'

::class NsDepClass public
::method describe
  return 'a namespace dep class'

::routine nsdeproutine public
  return 'nsdeproutine ran'
