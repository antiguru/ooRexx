/* What an internal package's routine contributes to a traceback.

   An argument error from one of these is reported under a line naming the
   routine, upcased whatever spelling the call used, and the major line names
   the package rather than the program and carries no line number. The error
   is left untrapped because trapping is what hides the report being pinned. */

say 'before   ' filespec('N', 'dir/leaf.txt')
say 'still    ' directory()  \== ''
say filespec('D')
