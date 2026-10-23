#ifndef APPKIT_CALLBACK_ABI_H
#define APPKIT_CALLBACK_ABI_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct {
    uint32_t version;
    size_t size;
} AppKitCallbackHeader;

typedef struct {
    AppKitCallbackHeader header;
    void (*deactivate)(uint64_t token);
    bool (*event)(uint64_t token, void *event);
    void (*text)(uint64_t token, const uint8_t *bytes, size_t length);
} AppKitCallbackTableV1;

typedef struct {
    size_t location;
    size_t length;
} AppKitUtf16Range;

#endif
