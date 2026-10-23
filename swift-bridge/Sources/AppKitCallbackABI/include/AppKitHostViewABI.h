#ifndef APPKIT_HOST_VIEW_ABI_H
#define APPKIT_HOST_VIEW_ABI_H

#include "AppKitCallbackABI.h"

typedef enum {
    AppKitHostViewPerformKeyEquivalent = 1,
    AppKitHostViewKeyDown = 2,
    AppKitHostViewKeyUp = 3,
    AppKitHostViewFlagsChanged = 4,
    AppKitHostViewLeftMouseDown = 5,
    AppKitHostViewLeftMouseUp = 6,
    AppKitHostViewRightMouseDown = 7,
    AppKitHostViewRightMouseUp = 8,
    AppKitHostViewOtherMouseDown = 9,
    AppKitHostViewOtherMouseUp = 10,
    AppKitHostViewLeftMouseDragged = 11,
    AppKitHostViewRightMouseDragged = 12,
    AppKitHostViewOtherMouseDragged = 13,
    AppKitHostViewMouseMoved = 14,
    AppKitHostViewMouseEntered = 15,
    AppKitHostViewMouseExited = 16,
    AppKitHostViewScrollWheel = 17,
    AppKitHostViewPressureChanged = 18,
    AppKitHostViewMagnified = 19,
    AppKitHostViewSwiped = 20
} AppKitHostViewEventSelector;

typedef enum {
    AppKitHostViewBackingPropertiesChanged = 1,
    AppKitHostViewFrameSizeChanged = 2,
    AppKitHostViewEffectiveAppearanceChanged = 3,
    AppKitHostViewDisplayLayer = 4,
    AppKitHostViewResetCursorRects = 5
} AppKitHostViewNotificationSelector;

typedef enum {
    AppKitHostViewAcceptsFirstMouse = 1,
    AppKitHostViewAcceptsFirstResponder = 2
} AppKitHostViewQuerySelector;

typedef struct {
    AppKitCallbackHeader header;
    void (*deactivate)(uint64_t token);
    bool (*event)(uint64_t token, uint32_t selector, void *event);
    void (*notification)(uint64_t token, uint32_t selector, double width, double height);
    bool (*query)(uint64_t token, uint32_t selector, void *event);
} AppKitHostViewCallbackTableV1;

#endif
