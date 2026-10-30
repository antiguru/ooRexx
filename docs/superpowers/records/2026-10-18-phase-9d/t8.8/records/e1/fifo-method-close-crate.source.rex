s=.Stream~new('/tmp/t88e1-probes-iphssopn/fifo-method-close-crate/pipe')
say 'write' s~lineout('hello')
say 'close' s~lineout
