r = .Routine~new('MYNAME', .array~of("f = .context~stackframes[1]", "return f~type f~name"))
say r~call
