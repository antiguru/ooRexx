r = .Routine~new("", .Array~of(  -
    '.c~r(.methods["FLOAT"])    ', -
    '::method float             ', -
    '::class c                  ', -
    '::method r class           ', -
    '    forward message "run"  ', -
    '::options trace "a"        '))
r~call
