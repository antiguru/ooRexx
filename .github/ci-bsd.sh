#!/bin/sh
# Build ooRexx and run its test suite inside a BSD VM.
#
# Shared by the FreeBSD and OpenBSD jobs. It lives in a file rather than inline
# in each job because GitHub Actions does not support YAML anchors, and the
# `uses:` of a VM action cannot be selected from a matrix, so the two jobs
# cannot be collapsed into one.
#
# Everything is written into the workspace, which the VM action rsyncs back to
# the host afterwards. The results are therefore judged on the host by the same
# script every other platform uses, rather than reimplemented here.

set -eux

cmake --version
uname -a

cmake -S . -B build -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_POLICY_VERSION_MINIMUM=3.5
cmake --build build --parallel 4

build/bin/rexx -v
echo 'say .rexxinfo~version' > hello.rex
build/bin/rexx hello.rex

svn checkout --non-interactive --trust-server-cert \
    https://svn.code.sf.net/p/oorexx/code-0/test/trunk ootest

# Run out of the build tree rather than an install, so the suite picks up the
# interpreter, the runtime libraries and the compiled native API test binaries
# together and the native API tests actually run.
WS=`pwd`
PATH="$WS/build/bin:$PATH"
LD_LIBRARY_PATH="$WS/build/lib:$WS/build/bin:${LD_LIBRARY_PATH:-}"
export PATH LD_LIBRARY_PATH

cd ootest
# The suite's exit code is recorded, not acted on. A non-zero code here is
# expected whenever an environmental test fails, and telling those apart from
# real ones is the host-side check step's job.
set +e
rexx testOORexx.rex -s > "$WS/testresults.txt" 2>&1
echo $? > "$WS/testexitcode.txt"
set -e

# The full file goes back to the host as an artifact; this is just enough to
# see what happened without downloading it.
tail -n 40 "$WS/testresults.txt"
