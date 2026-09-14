/* A condition the extension raised through RaiseException0 is raised in the
   caller's frame once the call has returned, so the traceback carries the
   native method's own line and the send that reached it. RegExp_Init raises
   38.0 for a template it cannot parse and then runs on to store its CSELF. */
r = .Re~new('[')
say 'never printed' r

::class Re subclass Object
::method init external "LIBRARY rxregexp RegExp_Init"
::method uninit external "LIBRARY rxregexp RegExp_Uninit"
::method doparse external "LIBRARY rxregexp RegExp_Parse"
