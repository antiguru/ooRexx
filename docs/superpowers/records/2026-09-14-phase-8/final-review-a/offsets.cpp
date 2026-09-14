// Prints sizes, alignments and offsets of the frozen header's structs, compiled
// as C++ so the __cplusplus branch is the one measured.
#include <cstdio>
#include <cstddef>
#include "oorexxapi.h"

#define Z(S) printf("%s size %zu align %zu\n", #S, sizeof(S), alignof(S))
#define P(S, F) printf("%s.%s %zu\n", #S, #F, offsetof(S, F))

struct MethodWrapper { RexxMethodContext c; void *owner; };
struct ThreadWrapper { RexxThreadContext c; void *owner; };

int main() {
    Z(RexxInstance_); P(RexxInstance_, functions); P(RexxInstance_, applicationData);
    Z(RexxThreadContext_); P(RexxThreadContext_, instance); P(RexxThreadContext_, functions);
    Z(RexxMethodContext_); P(RexxMethodContext_, threadContext); P(RexxMethodContext_, functions); P(RexxMethodContext_, arguments);
    Z(RexxCallContext_); P(RexxCallContext_, threadContext); P(RexxCallContext_, functions); P(RexxCallContext_, arguments);
    Z(RexxExitContext_); P(RexxExitContext_, threadContext); P(RexxExitContext_, functions); P(RexxExitContext_, arguments);
    Z(RexxIORedirectorContext_); P(RexxIORedirectorContext_, functions);
    Z(ValueDescriptor); P(ValueDescriptor, value); P(ValueDescriptor, type); P(ValueDescriptor, flags);
    Z(RexxCondition); P(RexxCondition, code); P(RexxCondition, rc); P(RexxCondition, position);
    P(RexxCondition, conditionName); P(RexxCondition, message); P(RexxCondition, errortext);
    P(RexxCondition, program); P(RexxCondition, description); P(RexxCondition, additional);
    Z(RexxPackageEntry); P(RexxPackageEntry, size); P(RexxPackageEntry, apiVersion); P(RexxPackageEntry, requiredVersion);
    P(RexxPackageEntry, packageName); P(RexxPackageEntry, packageVersion); P(RexxPackageEntry, loader);
    P(RexxPackageEntry, unloader); P(RexxPackageEntry, routines); P(RexxPackageEntry, methods);
    Z(RexxMethodEntry); P(RexxMethodEntry, style); P(RexxMethodEntry, reserved1); P(RexxMethodEntry, name);
    P(RexxMethodEntry, entryPoint); P(RexxMethodEntry, reserved2); P(RexxMethodEntry, reserved3);
    Z(RexxRoutineEntry); P(RexxRoutineEntry, style); P(RexxRoutineEntry, reserved1); P(RexxRoutineEntry, name);
    P(RexxRoutineEntry, entryPoint); P(RexxRoutineEntry, reserved2); P(RexxRoutineEntry, reserved3);
    Z(RexxInstanceInterface); P(RexxInstanceInterface, interfaceVersion); P(RexxInstanceInterface, AddCommandEnvironment);
    Z(RexxThreadInterface); P(RexxThreadInterface, interfaceVersion); P(RexxThreadInterface, WholeNumberToObject);
    P(RexxThreadInterface, StringLength); P(RexxThreadInterface, StringData); P(RexxThreadInterface, NewPointer);
    P(RexxThreadInterface, RaiseException0); P(RexxThreadInterface, RexxNil); P(RexxThreadInterface, RexxTrue);
    P(RexxThreadInterface, RexxFalse); P(RexxThreadInterface, RexxNullString); P(RexxThreadInterface, GetInterpreterInstance);
    Z(MethodContextInterface); P(MethodContextInterface, interfaceVersion); P(MethodContextInterface, SetObjectVariable);
    P(MethodContextInterface, DropObjectVariable); P(MethodContextInterface, ThrowCondition);
    Z(CallContextInterface); P(CallContextInterface, interfaceVersion); P(CallContextInterface, ThrowCondition);
    Z(ExitContextInterface); P(ExitContextInterface, interfaceVersion); P(ExitContextInterface, ThrowCondition);
    Z(IORedirectorInterface); P(IORedirectorInterface, interfaceVersion); P(IORedirectorInterface, IsRedirectionRequested);
    Z(MethodWrapper); P(MethodWrapper, owner);
    Z(ThreadWrapper); P(ThreadWrapper, owner);
    ValueDescriptor v;
    printf("union.value_int %zu\n", sizeof(v.value.value_int));
    printf("union.value_int8_t %zu\n", sizeof(v.value.value_int8_t));
    printf("union.value_int16_t %zu\n", sizeof(v.value.value_int16_t));
    printf("union.value_int32_t %zu\n", sizeof(v.value.value_int32_t));
    printf("union.value_int64_t %zu\n", sizeof(v.value.value_int64_t));
    printf("union.value_uint8_t %zu\n", sizeof(v.value.value_uint8_t));
    printf("union.value_uint16_t %zu\n", sizeof(v.value.value_uint16_t));
    printf("union.value_uint32_t %zu\n", sizeof(v.value.value_uint32_t));
    printf("union.value_uint64_t %zu\n", sizeof(v.value.value_uint64_t));
    printf("union.value_wholenumber_t %zu\n", sizeof(v.value.value_wholenumber_t));
    printf("union.value_stringsize_t %zu\n", sizeof(v.value.value_stringsize_t));
    printf("union.value_logical_t %zu\n", sizeof(v.value.value_logical_t));
    printf("union.value_double %zu\n", sizeof(v.value.value_double));
    printf("union.value_float %zu\n", sizeof(v.value.value_float));
    printf("union.value_CSTRING %zu\n", sizeof(v.value.value_CSTRING));
    printf("union.value_POINTER %zu\n", sizeof(v.value.value_POINTER));
    printf("union.value_RexxObjectPtr %zu\n", sizeof(v.value.value_RexxObjectPtr));
    printf("REXX_CURRENT_INTERPRETER_VERSION %d\n", REXX_CURRENT_INTERPRETER_VERSION);
    printf("REXX_PACKAGE_API_NO %d\n", REXX_PACKAGE_API_NO);
    printf("versions %d %d %d %d %d %d\n", INSTANCE_INTERFACE_VERSION, THREAD_INTERFACE_VERSION, METHOD_INTERFACE_VERSION,
           CALL_INTERFACE_VERSION, EXIT_INTERFACE_VERSION, REDIRECT_INTERFACE_VERSION);
    return 0;
}
