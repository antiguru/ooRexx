s=.Stream~new('/tmp/t88e1-probes-iphssopn/fifo-method-close-oracle/pipe')
say 'write' s~lineout('hello')
say 'close' s~lineout
