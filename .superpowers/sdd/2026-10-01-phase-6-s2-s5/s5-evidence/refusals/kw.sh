cd /home/moritz/dev/repos/ooRexx-rust-rewrite
X=docs/superpowers/plans/phase-4-exclusions.txt
# collapse wrapped lines so phrases split across lines are found
for k in 'invert' 'immovable' 'wrapper' 'busy' 'pinned' 'nested scheduler' 'halt.\{0,40\}nest' 'nohup' 'callback' 'baton' 'reply' 'phase 6' 'p6-' 'spec 2026-09-29' 'owner: none'; do
  echo "== $k"; /bin/grep -n -i -- "$k" $X | cut -c1-170 | head -15
done
