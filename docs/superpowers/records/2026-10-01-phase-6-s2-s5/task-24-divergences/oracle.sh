# usage: oracle.sh FILE  -- runs the oracle from a fresh empty dir under the rules' wrapper
f=$(realpath "$1")
d=$(mktemp -d)
trap 'rm -f "$d/p.rex" "$d/out" "$d/err"; rmdir "$d"' EXIT
cp "$f" "$d/p.rex"
cd "$d"
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib${EXTRA_LIB:+:$EXTRA_LIB} timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx p.rex ) >out 2>err
echo "rc=$?"
cat out
sed "s#$d/##; s/^/E: /" err
