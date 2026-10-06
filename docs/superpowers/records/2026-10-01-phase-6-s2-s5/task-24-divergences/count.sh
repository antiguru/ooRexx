# usage: count.sh oracle|ours FILE N -- tallies (stdout|rc|stderr) over N runs
w=$(dirname "$0")/$1.sh
for i in $(seq 1 "$3"); do bash "$w" "$2" | tr '\n' '|'; echo; done | sort | uniq -c
