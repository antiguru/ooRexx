#!/bin/sh
# Build ooRexx and run its test suite inside a Solaris VM.
#
# This is the BSD script's Solaris sibling. It is a separate file rather than a
# reuse of ci-bsd.sh because the pieces that differ are exactly the pieces a
# shared script cannot hide: Solaris has no ninja in IPS, so the build uses the
# Unix Makefiles generator driven by gmake; Solaris su is "su - user -c", not
# the GNU "su -l"; and Solaris chmod has no capital X. Everything else mirrors
# the BSD script so the two platforms are judged the same way.
#
# First-attempt notes are inline where the package names or paths are the most
# likely thing to need a second CI iteration.

set -eux

cmake --version
uname -a

# gmake is the real build tool; Solaris /usr/bin/make is not what cmake's
# generated makefiles expect. Prefer an explicit gmake if one is on PATH.
GMAKE=`command -v gmake || echo make`

# Cores enabled so a suite crash leaves a backtrace, the same as the BSD job.
ulimit -c unlimited || true

cmake -S . -B build -G "Unix Makefiles" \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_MAKE_PROGRAM="$GMAKE" \
    -DCMAKE_POLICY_VERSION_MINIMUM=3.5
cmake --build build --parallel 4

WS=`pwd`

build/bin/rexx -v
echo 'say .rexxinfo~version' > hello.rex
build/bin/rexx hello.rex

svn checkout --non-interactive --trust-server-cert \
    https://svn.code.sf.net/p/oorexx/code-0/test/trunk ootest

# The suite is not meant to run as root (its ReadMe.first says so, and root
# defeats every permission test), so it runs as an unprivileged user, the same
# as every other platform. Solaris useradd puts the home under /export/home.
TESTUSER=oorexxtest
useradd -m -s /bin/sh "$TESTUSER" || true

# Build tree stays root-owned and read-only; the suite dir must be writable.
# Solaris chmod has no capital X, so add the traverse bit with lowercase x. The
# build tree is only read, so a+rx over it is safe.
chmod -R a+rx "$WS/build"
chown -R "$TESTUSER" ootest

# The test user has to be able to walk down to the workspace.
dir=$WS
while [ "$dir" != "/" ] && [ -n "$dir" ]; do
    chmod a+x "$dir" 2>/dev/null || true
    dir=`dirname "$dir"`
done

PATH="$WS/build/bin:$PATH"
LD_LIBRARY_PATH="$WS/build/lib:$WS/build/bin:${LD_LIBRARY_PATH:-}"
export PATH LD_LIBRARY_PATH

: > "$WS/testresults.txt"
: > "$WS/testexitcode.txt"
chown "$TESTUSER" "$WS/testresults.txt" "$WS/testexitcode.txt"

# su - starts a login shell and discards the environment, so the paths are set
# again on the far side in a script rather than inline, to avoid the quoting
# trap that empties PATH. The heredoc is unquoted so $WS expands now while \$?
# is left for the script.
cat > "$WS/run-suite.sh" <<SCRIPT
#!/bin/sh
cd "$WS/ootest" || exit 1
PATH="$WS/build/bin:\$PATH"
LD_LIBRARY_PATH="$WS/build/lib:$WS/build/bin"
export PATH LD_LIBRARY_PATH
"$WS/build/bin/rexx" testOORexx.rex -s < /dev/null
echo \$? > "$WS/testexitcode.txt"
SCRIPT
chmod 755 "$WS/run-suite.sh"

# Solaris su is "su - user -c", not the GNU "su -l".
set +e
su - "$TESTUSER" -c "$WS/run-suite.sh" 2>&1 | tee "$WS/testresults.txt"
set -e

# Best-effort backtrace of any core. Solaris ships mdb; gdb may be absent, and a
# missing debugger must not turn a crash into a different error.
set +e
cores=`find "$WS" /var/crash -name '*.core' -o -name 'core' 2>/dev/null | head -n 5`
if [ -n "$cores" ]; then
    debugger=`command -v gdb || command -v mdb`
    for core in $cores; do
        echo "================ core: $core"
        file "$core" || true
        if [ -n "$debugger" ]; then
            case "$debugger" in
            *mdb) "$debugger" "$WS/build/bin/rexx" "$core" <<'MDB' 2>&1 | head -n 200
::status
$C
::quit
MDB
                ;;
            *)    "$debugger" -batch -ex 'bt full' -ex 'thread apply all bt' \
                      "$WS/build/bin/rexx" "$core" 2>&1 | head -n 200 ;;
            esac
        else
            echo "no gdb or mdb available to read it"
        fi
    done
else
    echo "no core files found"
fi
set -e
