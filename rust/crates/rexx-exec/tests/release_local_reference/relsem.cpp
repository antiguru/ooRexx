#include "oorexxapi.h"

static RexxThreadContext *kept;
static RexxObjectPtr stashed;

// Releases one string twice, then answers it.
RexxRoutine0(RexxObjectPtr, RelTwice)
{
    RexxObjectPtr s = context->String("released twice");
    context->ReleaseLocalReference(s);
    context->ReleaseLocalReference(s);
    return s;
}

// Keeps a string's handle and this call's thread context for a later call.
RexxRoutine0(RexxObjectPtr, RelStash)
{
    kept = context->threadContext;
    stashed = context->String("stashed");
    return context->String("kept");
}

// Releases the handle an earlier call kept, through that call's context,
// which this call never held.
RexxRoutine0(RexxObjectPtr, RelStale)
{
    kept->ReleaseLocalReference(stashed);
    return context->String("stale released");
}

// Releases one string of two and makes and releases n more; answers the
// other, then the released one's length.
RexxRoutine1(RexxObjectPtr, RelOne, int, n)
{
    RexxObjectPtr gone = context->String("released among many");
    RexxObjectPtr held = context->String("held among many");
    context->ReleaseLocalReference(gone);
    for (int i = 0; i < n; i++) {
        context->ReleaseLocalReference(context->String("some string that is long enough to be boxed on the heap"));
    }
    return held;
}

// Answers the length of a string read after its release.
RexxRoutine0(size_t, RelLength)
{
    RexxStringObject s = context->String("read after release");
    context->ReleaseLocalReference(s);
    return context->StringLength(s);
}

RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(RelTwice, RelTwice),
    REXX_TYPED_ROUTINE(RelStash, RelStash),
    REXX_TYPED_ROUTINE(RelStale, RelStale),
    REXX_TYPED_ROUTINE(RelOne, RelOne),
    REXX_TYPED_ROUTINE(RelLength, RelLength),
    REXX_LAST_ROUTINE()
};
RexxPackageEntry relsem_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "relsem", "1.0.0", NULL, NULL, routines, NULL
};
OOREXX_GET_PACKAGE(relsem);
