#ifndef APPKIT_TEXT_INPUT_ABI_H
#define APPKIT_TEXT_INPUT_ABI_H

#include "AppKitCallbackABI.h"
#include <CoreGraphics/CoreGraphics.h>

typedef enum {
    AppKitTextInputHasMarkedText = 1,
    AppKitTextInputMarkedRange = 2,
    AppKitTextInputSelectedRange = 3,
    AppKitTextInputFirstRect = 4,
    AppKitTextInputAttributedSubstring = 5,
    AppKitTextInputCharacterIndex = 6
} AppKitTextInputQuery;

typedef enum {
    AppKitTextInputInsertText = 1,
    AppKitTextInputSetMarkedText = 2,
    AppKitTextInputUnmarkText = 3
} AppKitTextInputAction;

typedef enum {
    AppKitTextInputNotConsumed = 0,
    AppKitTextInputConsumed = 1,
    AppKitTextInputHandledCommand = 2,
    AppKitTextInputUnhandledCommand = 3
} AppKitTextInputEventStatus;

typedef struct {
    AppKitUtf16Range range;
    CGRect rect;
    size_t character_index;
    bool has_marked_text;
} AppKitTextInputQueryResult;

typedef bool (*AppKitTextInputCopyUTF8)(void *context, const uint8_t *bytes, size_t length);

typedef struct {
    AppKitCallbackHeader header;
    void (*deactivate)(uint64_t token);
    bool (*query)(uint64_t token, uint32_t selector, AppKitUtf16Range range,
                  double x, double y, AppKitTextInputQueryResult *result,
                  void *context, AppKitTextInputCopyUTF8 copy_utf8);
    void (*action)(uint64_t token, uint32_t selector, const uint8_t *bytes,
                   size_t length, AppKitUtf16Range selection,
                   AppKitUtf16Range replacement);
    bool (*command)(uint64_t token, const uint8_t *bytes, size_t length);
} AppKitTextInputCallbackTableV1;

#endif
