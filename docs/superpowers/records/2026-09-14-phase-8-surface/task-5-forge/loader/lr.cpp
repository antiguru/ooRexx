#include "oorexxapi.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
static char dlog[256];
static void L(const char *s) { strncat(dlog, s, sizeof(dlog) - strlen(dlog) - 2); strcat(dlog, ";"); }
#ifndef SECOND
RexxRoutine0(RexxStringObject, LDtors) { RexxStringObject s = context->String(dlog); dlog[0] = 0; return s; }
RexxRoutine1(int, LoadIt, CSTRING, name) {
    logical_t p = context->LoadLibrary(name);
    L(p ? "loaded" : "load-null");
    L(context->CheckCondition() ? "held" : "clear");
    return 3;
}
RexxRoutineEntry routines[] = { REXX_TYPED_ROUTINE(LDtors, LDtors), REXX_TYPED_ROUTINE(LoadIt, LoadIt), REXX_LAST_ROUTINE() };
RexxPackageEntry lr_package_entry = { STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0, "lr", "1.0", NULL, NULL, routines, NULL };
OOREXX_GET_PACKAGE(lr);
#else
static void RexxEntry loader(RexxThreadContext *c) {
    fprintf(stderr, "loader raising\n");
    c->RaiseException0(40001);
    fprintf(stderr, "loader after raise\n");
}
RexxPackageEntry lr2_package_entry = { STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0, "lr2", "1.0", loader, NULL, NULL, NULL };
OOREXX_GET_PACKAGE(lr2);
#endif
