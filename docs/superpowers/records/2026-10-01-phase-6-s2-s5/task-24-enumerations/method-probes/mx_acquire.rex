s = .mutexSemaphore~new; say "ok" s~acquire; say "ok" s~acquire(0); say "ok" s~release s~release s~release
