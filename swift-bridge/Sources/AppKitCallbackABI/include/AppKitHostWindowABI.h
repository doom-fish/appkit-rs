#ifndef APPKIT_HOST_WINDOW_ABI_H
#define APPKIT_HOST_WINDOW_ABI_H

#include "AppKitCallbackABI.h"

#define APPKIT_HOST_WINDOW_EVENT_CLOSE 1
#define APPKIT_HOST_WINDOW_EVENT_WILL_CLOSE 2
#define APPKIT_HOST_WINDOW_EVENT_DID_RESIZE 3
#define APPKIT_HOST_WINDOW_EVENT_DID_MOVE 4
#define APPKIT_HOST_WINDOW_EVENT_DID_CHANGE_SCREEN 5
#define APPKIT_HOST_WINDOW_EVENT_DID_CHANGE_BACKING_PROPERTIES 6
#define APPKIT_HOST_WINDOW_EVENT_DID_BECOME_KEY 7
#define APPKIT_HOST_WINDOW_EVENT_DID_RESIGN_KEY 8
#define APPKIT_HOST_WINDOW_EVENT_DID_CHANGE_OCCLUSION_STATE 9
#define APPKIT_HOST_WINDOW_EVENT_WILL_ENTER_FULL_SCREEN 10
#define APPKIT_HOST_WINDOW_EVENT_DID_ENTER_FULL_SCREEN 11
#define APPKIT_HOST_WINDOW_EVENT_DID_FAIL_TO_ENTER_FULL_SCREEN 12
#define APPKIT_HOST_WINDOW_EVENT_WILL_EXIT_FULL_SCREEN 13
#define APPKIT_HOST_WINDOW_EVENT_DID_EXIT_FULL_SCREEN 14
#define APPKIT_HOST_WINDOW_EVENT_DID_FAIL_TO_EXIT_FULL_SCREEN 15
#define APPKIT_HOST_WINDOW_EVENT_ADDED_TITLEBAR_ACCESSORY 16
#define APPKIT_HOST_WINDOW_EVENT_MOVE_TAB_TO_NEW_WINDOW 17
#define APPKIT_HOST_WINDOW_EVENT_MERGE_ALL_WINDOWS 18
#define APPKIT_HOST_WINDOW_EVENT_SELECT_NEXT_TAB 19
#define APPKIT_HOST_WINDOW_EVENT_SELECT_PREVIOUS_TAB 20
#define APPKIT_HOST_WINDOW_EVENT_TOGGLE_TAB_BAR 21

typedef struct {
    AppKitCallbackHeader header;
    void (*deactivate)(uint64_t token);
    bool (*should_close)(uint64_t token);
    void (*event)(uint64_t token, uint32_t event);
} AppKitHostWindowCallbackTableV1;

typedef struct {
    double x;
    double y;
    double width;
    double height;
    uint64_t style_mask;
    int64_t backing_store;
    void *screen;
    bool defer_creation;
    bool can_become_main;
    bool can_become_key;
} AppKitHostedWindowConfiguration;

typedef struct {
    int8_t is_floating_panel;
    int8_t becomes_key_only_if_needed;
    int8_t works_when_modal;
    int8_t hides_on_deactivate;
} AppKitHostedPanelOptions;

#endif
