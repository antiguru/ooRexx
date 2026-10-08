/* A ::REQUIRES file that does not parse raises SYNTAX in that file, its
   clause above the directive's line. */
say 'not reached'
::requires 'badreq.cls'
