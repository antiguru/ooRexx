s=.Stream~new('/tmp/t88e1-probes-fb0srth_/fifo-method-close-oracle/pipe')
say 'write' s~lineout('hello')
say 'close' s~lineout
