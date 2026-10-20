address system 'true'
interpret "address system 'exit 4'; say 'in' .rs .context~rs rc"
say 'out' .rs .context~rs rc
c = .context
interpret "address system 'exit 127'; say 'in2' .rs c~rs .context~rs (c == .context)"
say 'out2' .rs .context~rs
