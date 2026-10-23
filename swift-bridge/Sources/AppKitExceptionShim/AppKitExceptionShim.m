#import "AppKitExceptionShim.h"

NSString *_Nullable appkit_try(NS_NOESCAPE void (^block)(void)) {
    @try {
        block();
        return nil;
    } @catch (NSException *exception) {
        return [NSString stringWithFormat:@"%@: %@", exception.name, exception.reason ?: @""];
    }
}
