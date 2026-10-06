s = .eventSemaphore~new; say "ok" s~wait(0); s~post; say "ok" s~wait; say "ok" s~wait(1)
