# usage: REXX_RUN=<release rexx-run> ours.sh FILE  -- runs this crate from a fresh empty dir
f=$(realpath "$1")
d=$(mktemp -d)
trap 'rm -f "$d/p.rex" "$d/out" "$d/err"; rmdir "$d"' EXIT
cp "$f" "$d/p.rex"
cd "$d"
LD_LIBRARY_PATH=${EXTRA_LIB:+$EXTRA_LIB:}$LD_LIBRARY_PATH timeout -s KILL 20 "$REXX_RUN" p.rex >out 2>err
echo "rc=$?"
cat out
sed "s#$d/##; s/^/E: /" err
