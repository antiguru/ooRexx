/* Called by package_default_options.rex: what this file's own package took. */
pk = .context~package
return digits() form() fuzz() trace() pk~options('NU') pk~options('NOV') pk~options('P'),
  '[' || pk~options('X') || ']'
::options fuzz 2 trace n
