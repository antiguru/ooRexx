/* ::OPTIONS NOVALUE ERROR reaches a ::CONSTANT expression: the unassigned
   variable is 98.986 at install, before the prolog runs, and not the 41.1 of
   adding its name. */
say 'prolog'

::class c
::constant k (1 + b)

::options novalue error
