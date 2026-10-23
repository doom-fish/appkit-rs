#ifndef APPKIT_APPLICATION_HOST_ABI_H
#define APPKIT_APPLICATION_HOST_ABI_H

#include "AppKitCallbackABI.h"

typedef struct {
    size_t offset;
    size_t length;
} AppKitApplicationHostUtf8Range;

typedef struct AppKitApplicationHostRetainedMenu AppKitApplicationHostRetainedMenu;

typedef struct {
    AppKitCallbackHeader header;
    void (*deactivate)(uint64_t token);
    void (*will_finish_launching)(uint64_t token);
    void (*did_finish_launching)(uint64_t token);
    bool (*should_handle_reopen)(uint64_t token, bool has_visible_windows);
    bool (*should_terminate)(uint64_t token);
    void (*will_terminate)(uint64_t token);
    void (*open_urls)(uint64_t token, const uint8_t *bytes, size_t byte_length, const AppKitApplicationHostUtf8Range *ranges, size_t count);
    AppKitApplicationHostRetainedMenu *(*dock_menu_retained)(uint64_t token);
    void (*menu_item_action)(uint64_t token, int64_t tag, int32_t action);
    bool (*validate_menu_item)(uint64_t token, int64_t tag, int32_t action);
    void (*menu_will_open)(uint64_t token);
    void (*screen_parameters_changed)(uint64_t token);
} AppKitApplicationHostCallbacksV1;

#endif
