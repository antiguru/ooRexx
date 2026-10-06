# usage: run30.sh file.rex outdir N
F=$1; O=$2; N=$3; S=/tmp/claude-1000/p6-s5-refusals
mkdir -p $O
for i in $(seq 1 $N); do
  d=$(mktemp -d $S/run.XXXXXX); cp $F $d/p.rex; cd $d
  ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx p.rex ) > $O/oracle.$i 2>&1; echo "rc $?" >> $O/oracle.$i
  timeout -k 5 20 $S/target/release/rexx-run p.rex > $O/ours.$i 2>&1; echo "rc $?" >> $O/ours.$i
  cd $S; rm -f $d/p.rex; rmdir $d
done
echo "oracle distinct outputs:"; md5sum $O/oracle.* | awk '{print $1}' | sort | uniq -c
echo "ours distinct outputs:"; md5sum $O/ours.* | awk '{print $1}' | sort | uniq -c
