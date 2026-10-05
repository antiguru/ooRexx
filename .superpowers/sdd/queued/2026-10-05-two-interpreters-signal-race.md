# Two interpreters in one process reopen the Ctrl-C command race

Found by the Phase 6 S2-S5 Task 21 re-review 3 (F-3). For the interpreter API phase.

P67 (amended) closes the race where a group SIGINT's command end beats the halt only for one
interpreter per process: the handler runs on the interpreter thread or the waiting pool thread. With
two interpreters in one embedder process a group Ctrl-C halted the next command 11/120 under load
(0/60 with one). Direction: consult `sigpending` before filing a command's end, or route the signal to
the waiting thread. Also: `install_signal_handlers` blocks the halting signals in its caller's thread,
which an embedder must know.
