# usage: oracle.sh FILE  -- runs the oracle from a fresh empty dir
f=$(realpath "$1")
d=$(mktemp -d /tmp/claude-1000/p6-t24/run.XXXXXX)
cp "$f" "$d/p.rex"
cd "$d"
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -s KILL -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx p.rex ) >out 2>err
echo "rc=$?"
cat out
sed 's/^/E: /' err
