#include "oorexxapi.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <stdint.h>
#ifndef SECOND
struct Noisy { const char *n; ~Noisy() { fprintf(stderr, "dtor %s\n", n); } };
RexxRoutine0(int, LoadThrow2) {
    char b[64]; snprintf(b, sizeof b, "%p", (void *)context); setenv("LT_CTX", b, 1);
    Noisy n = {"loadthrow2"};
    logical_t p = context->LoadLibrary("lt2");
    fprintf(stderr, "after LoadLibrary %d held=%d\n", (int)p, (int)context->CheckCondition());
    return 3;
}
RexxRoutineEntry routines[] = { REXX_TYPED_ROUTINE(LoadThrow2, LoadThrow2), REXX_LAST_ROUTINE() };
RexxPackageEntry lt_package_entry = { STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0, "lt", "1.0", NULL, NULL, routines, NULL };
OOREXX_GET_PACKAGE(lt);
#else
static void RexxEntry loader(RexxThreadContext *) {
    RexxCallContext *c = (RexxCallContext *)(uintptr_t)strtoull(getenv("LT_CTX"), NULL, 16);
    fprintf(stderr, "loader throwing\n");
    c->ThrowException0(40001);
    fprintf(stderr, "loader after throw\n");
}
RexxPackageEntry lt2_package_entry = { STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0, "lt2", "1.0", loader, NULL, NULL, NULL };
OOREXX_GET_PACKAGE(lt2);
#endif
