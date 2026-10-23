#ifndef APPKIT_EVENT_ABI_H
#define APPKIT_EVENT_ABI_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
    uint8_t *bytes;
    size_t length;
} AppKitEventString;

#endif
