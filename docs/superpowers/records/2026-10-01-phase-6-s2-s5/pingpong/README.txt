# Criterion 7, from rust/, rexx-run a release build of e57dc8315 without features:
# PROGRAMS="pingpong/pingmsg pingpong/pingsem pingpong/pingguard" bash bench-programs/wallclock.sh -r 9 -o OUT -x "pingpong/pingmsg pingpong/pingsem pingpong/pingguard" rexx-run=TARGET/release/rexx-run
# wallclock.sh exited 0: every run exited 0 and every stdout matched rexx-run's first.
# 32 CPUs; load averages before and after are in binaries.txt.
