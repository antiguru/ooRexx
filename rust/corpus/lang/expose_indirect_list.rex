/* The EXPOSE (list) form is plural and ordered: the selector's own name is
   bound first and its value is then read out of the pool, which is what
   decides the rest of the list. LISTER holds BETA in the pool, so BETA is
   what gets exposed -- reading the selector out of the frame instead would
   spell the list from its own derived name. */
say .K~setlister
say .K~setbeta
say .K~indirect

::class K

::method setlister class
  expose lister
  lister = 'BETA'
  return 'lister set'

::method setbeta class
  expose beta
  beta = 'beta-value'
  return 'beta set'

::method indirect class
  expose (lister)
  return '['lister']['beta']'
