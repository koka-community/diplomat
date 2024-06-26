#ifndef diplomat_result_double_void_H
#define diplomat_result_double_void_H
#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#ifdef __cplusplus
namespace capi {
extern "C" {
#endif
typedef struct diplomat_result_double_void {
    union {
        double ok;
    };
    bool is_ok;
} diplomat_result_double_void;
#ifdef __cplusplus
} // extern "C"
} // namespace capi
#endif
#endif
