#ifndef APPKIT_ACCESSIBILITY_ABI_H
#define APPKIT_ACCESSIBILITY_ABI_H

#include "AppKitCallbackABI.h"

typedef int32_t (*AppKitAccessibilityNodeString)(void * _Nonnull, int32_t, uintptr_t, uintptr_t, const uint8_t * _Nullable * _Nonnull, uintptr_t * _Nonnull);
typedef bool (*AppKitAccessibilityNodeNumber)(void * _Nonnull, int32_t, double * _Nonnull);
typedef bool (*AppKitAccessibilityNodeInteger)(void * _Nonnull, int32_t, intptr_t, intptr_t * _Nonnull);
typedef bool (*AppKitAccessibilityNodeFlag)(void * _Nonnull, int32_t, int32_t);
typedef int32_t (*AppKitAccessibilityNodeValue)(void * _Nonnull, double * _Nonnull, const uint8_t * _Nullable * _Nonnull, uintptr_t * _Nonnull);
typedef int32_t (*AppKitAccessibilityNodeList)(void * _Nonnull, int32_t, void * _Nullable const * _Nullable * _Nonnull, uintptr_t * _Nonnull);
typedef int32_t (*AppKitAccessibilityNodeRect)(void * _Nonnull, int32_t, uintptr_t, uintptr_t, double * _Nonnull);
typedef bool (*AppKitAccessibilityNodeRange)(void * _Nonnull, int32_t, intptr_t, double, double, uintptr_t * _Nonnull, uintptr_t * _Nonnull);
typedef void (*AppKitAccessibilityTextRun)(void * _Nullable, const uint8_t * _Nullable, uintptr_t, uint64_t, uint64_t, const uint8_t * _Nullable, uintptr_t, double, bool, bool, uint64_t, uint64_t, const uint8_t * _Nullable, uintptr_t, int32_t);
typedef bool (*AppKitAccessibilityNodeTextRuns)(void * _Nonnull, uintptr_t, uintptr_t, void * _Nullable, AppKitAccessibilityTextRun _Nonnull);
typedef bool (*AppKitAccessibilityNodeAction)(void * _Nonnull, int32_t, double, uintptr_t, uintptr_t, const uint8_t * _Nullable, uintptr_t);
typedef void (*AppKitAccessibilityNodeRelease)(void * _Nonnull);
typedef void (*AppKitAccessibilityViewChildren)(void * _Nonnull, void * _Nullable const * _Nullable * _Nonnull, uintptr_t * _Nonnull);
typedef void * _Nullable (*AppKitAccessibilityViewFocus)(void * _Nonnull);
typedef void * _Nullable (*AppKitAccessibilityViewHitTest)(void * _Nonnull, double, double);

typedef struct {
    AppKitCallbackHeader header;
    AppKitAccessibilityNodeString _Nullable string;
    AppKitAccessibilityNodeNumber _Nullable number;
    AppKitAccessibilityNodeInteger _Nullable integer;
    AppKitAccessibilityNodeFlag _Nullable flag;
    AppKitAccessibilityNodeValue _Nullable value;
    AppKitAccessibilityNodeList _Nullable list;
    AppKitAccessibilityNodeRect _Nullable rect;
    AppKitAccessibilityNodeRange _Nullable range;
    AppKitAccessibilityNodeTextRuns _Nullable textRuns;
    AppKitAccessibilityNodeAction _Nullable action;
    AppKitAccessibilityNodeRelease _Nullable release;
    AppKitAccessibilityViewChildren _Nullable viewChildren;
    AppKitAccessibilityViewFocus _Nullable viewFocus;
    AppKitAccessibilityViewHitTest _Nullable viewHitTest;
} AppKitAccessibilityCallbackTableV1;

#endif
