#ifndef APPKIT_ALERT_ABI_H
#define APPKIT_ALERT_ABI_H

#include "AppKitCallbackABI.h"

typedef struct {
    AppKitCallbackHeader header;
    void (*deactivate)(uint64_t token);
    void (*complete)(uint64_t token, int64_t button_index);
} AppKitAlertCallbacksV1;

#endif
