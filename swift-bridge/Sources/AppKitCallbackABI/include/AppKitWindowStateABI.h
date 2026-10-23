#ifndef APPKIT_WINDOW_STATE_ABI_H
#define APPKIT_WINDOW_STATE_ABI_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
    uint8_t *bytes;
    size_t length;
} AppKitWindowStateBytes;

#endif
