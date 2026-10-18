p = .package~new(filespec('location', .context~package~name)'../a/lib.cls')
say p~findRoutine('go')~call
