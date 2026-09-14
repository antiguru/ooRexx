/* A library a required package's ::METHOD names and nothing loads is
   reported against that package's own file and line, before either prolog
   runs. */
say 'main ran'

::requires 'pk.cls'
