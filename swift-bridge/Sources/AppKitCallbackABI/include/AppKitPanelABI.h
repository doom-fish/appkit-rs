#ifndef APPKIT_PANEL_ABI_H
#define APPKIT_PANEL_ABI_H

#include "AppKitCallbackABI.h"

typedef struct {
    const uint8_t *bytes;
    size_t length;
} AppKitFileSystemPath;

typedef struct {
    AppKitCallbackHeader header;
    void (*deactivate)(uint64_t token);
    void (*complete)(uint64_t token, bool accepted, const AppKitFileSystemPath *paths, size_t count);
} AppKitPanelCallbacksV1;

#endif
