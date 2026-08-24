/* Sending to a LIBRARY REXX entry point the phase that owns the external
   queues has not built. The bind succeeds, so 'main' prints and the refusal
   is the send's. */
say 'main'
say .k~probe

::class k

::method probe class external 'LIBRARY REXX rexx_query_queue'
