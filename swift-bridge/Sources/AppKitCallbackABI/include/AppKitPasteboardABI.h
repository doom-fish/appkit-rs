#ifndef APPKIT_PASTEBOARD_ABI_H
#define APPKIT_PASTEBOARD_ABI_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
    const uint8_t *bytes;
    size_t length;
} AppKitPasteboardInput;

typedef struct {
    uint8_t *bytes;
    size_t length;
} AppKitPasteboardBytes;

typedef struct {
    AppKitPasteboardBytes *items;
    size_t count;
} AppKitPasteboardBytesList;

#endif
