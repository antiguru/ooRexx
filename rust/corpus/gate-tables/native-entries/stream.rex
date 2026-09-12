/* Sending to a LIBRARY REXX entry point the stream subsystem does not build.
   The bind succeeds, so 'main' prints and the refusal is the send's.
   `handle_set` is the one stream entry point that stays deferred for the rest
   of this phase -- reaching an already-open descriptor needs `from_raw_fd` --
   so this program does not have to be repointed as the family fills in. */
say 'main'
say .k~probe

::class k

::method probe class external 'LIBRARY REXX handle_set'
