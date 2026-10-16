#!/usr/bin/env bash
# Usage: derive.sh SCRATCH_ROOT [CORPUS] > NEW_CORPUS
# Re-derives parse-errors.tsv from the oracle. CORPUS defaults to the
# parse-errors.tsv beside this script and may be in either column layout.
set -u

scratch=${1:?usage: derive.sh SCRATCH_ROOT [CORPUS]}
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
corpus=${2:-$here/parse-errors.tsv}
oracle=/home/moritz/dev/repos/ooRexx/build

# Prints `expect<TAB>line<TAB>message` for one rexxc stderr file, the message
# escaped as the program field is.
extract='
undef $/;
my $e = <STDIN>;
my ($sub, $msg, $at);
$sub = "$1.$2", $msg = $3 if $e =~ /^Error (\d+)\.(\d+):  ([^\n]*)$/m;
$at = $1 if $e =~ /^Error \d+ running .* line (\d+):/m;
if (!defined $sub) { print "ok\t-\t-"; exit }
die "no main line\n" unless defined $at;
$msg =~ s/\\/\\\\/g;
$msg =~ s/\t/\\t/g;
$msg =~ s/\r/\\r/g;
$msg =~ s/([^\x20-\x7e])/sprintf("\\x%02x", ord $1)/ge;
$msg = "-" if $msg eq "";
print "$sub\t$at\t$msg";
'

fail() {
    echo "derive.sh: $*" >&2
    exit 1
}

row=0
while IFS= read -r line || [ -n "$line" ]; do
    row=$((row + 1))
    case $line in
    '#'*)
        printf '%s\n' "$line"
        continue
        ;;
    class$'\t'*)
        printf 'class\texpect\tline\tmessage\tprogram\n'
        case $line in
        *$'\tmessage\t'*) layout=5 ;;
        *) layout=4 ;;
        esac
        continue
        ;;
    esac
    if [ "$layout" = 5 ]; then
        IFS=$'\t' read -r class expect at _old program <<<"$line"
    else
        IFS=$'\t' read -r class expect at program <<<"$line"
    fi

    dir=$scratch/row$row
    mkdir -p "$dir" || fail "row $row: cannot make $dir"
    printf '%b' "$program" >"$dir/p.rex"
    (
        cd "$dir" || exit 99
        ulimit -v 1048576
        LD_LIBRARY_PATH=$oracle/lib timeout -k 5 20 "$oracle/bin/rexxc" "$dir/p.rex" -s \
            >"$dir/out" 2>"$dir/err"
        echo $? >"$dir/rc"
    )
    rc=$(<"$dir/rc")
    [ "$rc" = 99 ] && fail "row $row: cannot enter $dir"
    got=$(LC_ALL=C perl -e "$extract" <"$dir/err" || fail "row $row: unreadable stderr")
    IFS=$'\t' read -r gexpect gat gmsg <<<"$got"

    if [ "$class" = '?' ]; then
        case $gexpect in
        ok) class=- ;;
        98.903 | 90.999) class=install ;;
        *) class=translation ;;
        esac
    fi
    [ "$expect" = '?' ] && expect=$gexpect
    [ "$at" = '?' ] && at=$gat
    [ "$expect" = "$gexpect" ] || fail "row $row: expect $expect, oracle $gexpect (rc $rc)"
    [ "$at" = "$gat" ] || fail "row $row: line $at, oracle $gat"
    if [ "$gexpect" = ok ]; then
        [ "$rc" = 0 ] || fail "row $row: accepted but rc $rc"
    else
        [ "$rc" != 0 ] || fail "row $row: rejected but rc 0"
    fi
    printf '%s\t%s\t%s\t%s\t%s\n' "$class" "$expect" "$at" "$gmsg" "$program"
done <"$corpus"
