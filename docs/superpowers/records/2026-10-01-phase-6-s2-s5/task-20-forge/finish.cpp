#include "oorexxapi.h"
#include <string.h>

RexxRoutine1(RexxObjectPtr, finishedinplace, CSTRING, text)
{
    size_t n = strlen(text);
    RexxBufferStringObject s = context->NewBufferString(n);
    const char *early = context->StringData((RexxStringObject)s);
    char *data = (char *)context->BufferStringData(s);
    memcpy(data, text, n);
    context->FinishBufferString(s, n);
    return context->NewString(early, n);
}

RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(FINISHEDINPLACE, finishedinplace),
    REXX_LAST_ROUTINE()
};

RexxPackageEntry finish_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "finish", "1.0", NULL, NULL, routines, NULL
};

OOREXX_GET_PACKAGE(finish);
