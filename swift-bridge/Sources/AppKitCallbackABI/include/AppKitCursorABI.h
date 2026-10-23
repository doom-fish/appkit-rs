#ifndef APPKIT_CURSOR_ABI_H
#define APPKIT_CURSOR_ABI_H

#include "AppKitCallbackABI.h"

typedef enum {
    AppKitCursorKindArrow = 0,
    AppKitCursorKindIBeam = 1,
    AppKitCursorKindIBeamForVerticalLayout = 2,
    AppKitCursorKindCrosshair = 3,
    AppKitCursorKindOpenHand = 4,
    AppKitCursorKindClosedHand = 5,
    AppKitCursorKindPointingHand = 6,
    AppKitCursorKindResizeLeftRight = 7,
    AppKitCursorKindResizeUpDown = 8,
    AppKitCursorKindResizeLeft = 9,
    AppKitCursorKindResizeRight = 10,
    AppKitCursorKindResizeUp = 11,
    AppKitCursorKindResizeDown = 12,
    AppKitCursorKindResizeColumn = 13,
    AppKitCursorKindResizeRow = 14,
    AppKitCursorKindResizeUpLeftDownRight = 15,
    AppKitCursorKindResizeUpRightDownLeft = 16,
    AppKitCursorKindOperationNotAllowed = 17,
    AppKitCursorKindDragLink = 18,
    AppKitCursorKindDragCopy = 19,
    AppKitCursorKindContextualMenu = 20
} AppKitCursorKind;

#endif
