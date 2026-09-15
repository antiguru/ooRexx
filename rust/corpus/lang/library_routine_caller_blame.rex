/* A routine no directive has bound has no package, so a raise the boundary
   makes for itself is reported at the calling clause, here in the required
   package whose routine made the call. */
say 'start'
say inner()
::requires 'pk.cls'
