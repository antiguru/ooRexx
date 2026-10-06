# Criterion 7, from rust/, rexx-run a release build of 0ac73b804 without features:
# PROGRAMS="pingmsg pingsem pingguard" bash bench-programs/wallclock.sh -r 9 -o OUT -x "pingmsg pingsem pingguard" rexx-run=TARGET/release/rexx-run
# wallclock.sh exited 0: every run exited 0 and every stdout matched rexx-run's first.
# 32 CPUs; load averages before and after are in binaries.txt.
