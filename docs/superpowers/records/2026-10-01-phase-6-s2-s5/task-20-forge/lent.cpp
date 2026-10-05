#include "oorexxapi.h"
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>

// rexx-api's await_file: up to five seconds for a file at path
static bool await_file(const char *path)
{
    struct stat st;
    for (int i = 0; i < 500; i++)
    {
        usleep(10000);
        if (stat(path, &st) == 0) return true;
    }
    return false;
}

// HOLDBUFFER(buffer, path, mark): rexx-api's hold_buffer
RexxRoutine3(RexxObjectPtr, holdbuffer, RexxMutableBufferObject, b, CSTRING, path, CSTRING, mark)
{
    char *data = (char *)context->MutableBufferData(b);
    size_t length = context->MutableBufferLength(b);
    char held[4096];
    snprintf(held, sizeof(held), "%s.held", path);
    FILE *f = fopen(held, "w");
    if (f != NULL) { fputs("held", f); fclose(f); }
    if (!await_file(path)) return context->String("unawaited");
    char *copy = new char[length + 1];
    memcpy(copy, data, length);
    size_t n = strlen(mark);
    memcpy(data, mark, n < length ? n : length);
    RexxObjectPtr r = context->NewString(copy, length);
    delete[] copy;
    return r;
}

// FINISHEDINPLACE(text, made): rexx-api's finished_in_place
RexxRoutine2(RexxObjectPtr, finishedinplace, CSTRING, text, size_t, made)
{
    size_t n = strlen(text);
    if (made < n) made = n;
    RexxBufferStringObject s = context->NewBufferString(made);
    const char *early = context->StringData((RexxStringObject)s);
    char *data = (char *)context->BufferStringData(s);
    memset(data, 'x', made);
    memcpy(data, text, n);
    context->FinishBufferString(s, n);
    return context->NewStringFromAsciiz(early);
}

RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(HOLDBUFFER, holdbuffer),
    REXX_TYPED_ROUTINE(FINISHEDINPLACE, finishedinplace),
    REXX_LAST_ROUTINE()
};

RexxPackageEntry lent_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "lent", "1.0", NULL, NULL, routines, NULL
};

OOREXX_GET_PACKAGE(lent);
