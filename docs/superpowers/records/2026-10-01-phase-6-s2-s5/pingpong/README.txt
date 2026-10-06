# Criterion 7, from rust/. rexx-run: release, without features, built from a
# git archive of d9e17e5c8 in its own target dir (the .text hash in
# ../sharing-off-text-hash.txt):
# PROGRAMS="pingpong/pingmsg pingpong/pingsem pingpong/pingguard" bash bench-programs/wallclock.sh -r 9 -o OUT -x "pingpong/pingmsg pingpong/pingsem pingpong/pingguard" rexx-run=REXX_RUN
# wallclock.sh exited 0: every run exited 0 and every stdout matched rexx-run's first.
# 32 CPUs; load averages before and after are in binaries.txt.
