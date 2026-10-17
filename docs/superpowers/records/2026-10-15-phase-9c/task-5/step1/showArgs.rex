-- program to display various items related to its invocation
    say 'The arguments string is ['arg(1)']'
    argcnt = .syscargs~items
    if argcnt = 1 then
        say 'There is 1 entry in .SYSCARGS:'
    else
        say 'There are' argcnt 'entries in .SYSCARGS:'
    say '['.syscargs~toString(, ", ")']'
