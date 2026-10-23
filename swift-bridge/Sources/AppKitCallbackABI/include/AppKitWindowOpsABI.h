#ifndef APPKIT_WINDOW_OPS_ABI_H
#define APPKIT_WINDOW_OPS_ABI_H

#include "AppKitCallbackABI.h"

typedef struct {
    double red;
    double green;
    double blue;
    double alpha;
} AppKitRGBA;

typedef struct {
    AppKitCallbackHeader header;
    void (*deactivate)(uint64_t token);
    void (*complete)(uint64_t token, int64_t response);
} AppKitWindowSheetCallbackTableV1;

#endif
