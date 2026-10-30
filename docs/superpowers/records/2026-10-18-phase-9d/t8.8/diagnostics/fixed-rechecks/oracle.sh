#!/bin/bash
ulimit -v 1048576
export LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
exec timeout -k 2s 20s /home/moritz/dev/repos/ooRexx/build/bin/rexx "$@"
