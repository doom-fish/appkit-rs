#ifndef APPKIT_DRAG_ABI_H
#define APPKIT_DRAG_ABI_H

#include "AppKitCallbackABI.h"

#define APPKIT_DRAG_ENTERED 1
#define APPKIT_DRAG_UPDATED 2
#define APPKIT_DRAG_EXITED 3
#define APPKIT_DRAG_PERFORMED 4
#define APPKIT_DRAG_CONCLUDED 5

#define APPKIT_DRAG_FILE 1
#define APPKIT_DRAG_TEXT 2

typedef struct {
    const uint8_t *bytes;
    size_t length;
} AppKitDragPath;

typedef struct {
    double x;
    double y;
    uint64_t source_operation_mask;
    bool is_source_window;
    bool has_paths;
    const AppKitDragPath *paths;
    size_t path_count;
} AppKitDragInfo;

typedef struct {
    AppKitCallbackHeader header;
    void (*deactivate)(uint64_t token);
    uint64_t (*destination)(uint64_t token, uint32_t phase, const AppKitDragInfo *info);
    uint64_t (*source_mask)(uint64_t token, int64_t context);
    void (*source_end)(uint64_t token, double x, double y, uint64_t operation);
} AppKitDragCallbackTableV1;

typedef struct {
    uint32_t kind;
    const uint8_t *bytes;
    size_t length;
    bool is_directory;
} AppKitDragItem;

#endif
