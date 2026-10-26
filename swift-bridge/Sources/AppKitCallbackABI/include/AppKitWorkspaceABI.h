#ifndef APPKIT_WORKSPACE_ABI_H
#define APPKIT_WORKSPACE_ABI_H

#include "AppKitCallbackABI.h"

typedef struct {
    const char *bytes;
} AppKitWorkspacePath;

typedef struct {
    AppKitCallbackHeader header;
    void (*event)(uint64_t token, int32_t kind);
} AppKitWorkspacePowerTableV1;

typedef struct {
    AppKitCallbackHeader header;
    void (*changed)(uint64_t token);
} AppKitWorkspaceAccessibilityDisplayTableV1;

typedef struct {
    AppKitCallbackHeader header;
    void (*complete)(uint64_t token, bool success, const uint8_t *bytes, size_t length);
} AppKitWorkspaceSchemeTableV1;

#endif
