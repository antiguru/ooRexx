#!/bin/sh
# usage: mktree.sh DIR  -- builds the probe tree under DIR/sysfiletree
set -e
r="$1/sysfiletree"
mkdir -p "$r"
cd "$r"
: > f; printf x > .h; head -c 999 /dev/zero > 'b b'; printf xx > UpperFile
mkdir UpperDir d .d
head -c 10 /dev/zero > d/F; head -c 11 /dev/zero > d/.h
head -c 20 /dev/zero > .d/f; head -c 21 /dev/zero > .d/.h
printf 'y' > "$(printf 'n\377')"
printf 'z' > "$(printf 'n\357\277\275')"
ln -s d linkdir
ln -s nowhere dangling
mkdir locked; : > locked/inner
: > sticky; : > suid; : > sgid
mkfifo pipe
find . -exec touch -h -d '2026-10-13 08:09:10' {} +
touch -h -d '1890-01-02 03:04:05' f
touch -d '2026-10-12 00:00:00' .h
touch -d '2026-10-12 12:00:00' 'b b'
touch -d '2026-10-12 23:59:59' d/F
touch -d '2001-02-03 13:05:06' UpperDir
touch -d '2026-10-12 13:00:00' d
touch -h -d '2026-10-11 01:02:03' . 
chmod a-w UpperFile
chmod 1757 sticky; chmod 4755 suid; chmod 2645 sgid
chmod 000 locked
