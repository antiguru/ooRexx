f=$(realpath "$1")
d=$(mktemp -d /tmp/claude-1000/p6-t24/run.XXXXXX)
cp "$f" "$d/p.rex"
cd "$d"
timeout -s KILL 20 /tmp/claude-1000/p6-t24/target/release/rexx-run p.rex >out 2>err
echo "rc=$?"
cat out
sed 's/^/E: /' err
