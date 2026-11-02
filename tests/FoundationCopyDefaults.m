/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
// Native Foundation reference fixture, not a historical iPhone OS runtime.
#import <Foundation/Foundation.h>
#include <assert.h>
#include <stdio.h>

static unsigned liveTokens;
@interface CopyToken : NSObject
@end
@implementation CopyToken
- (id)init {
    self = [super init];
    if (self) liveTokens++;
    return self;
}
- (void)dealloc {
    liveTokens--;
    [super dealloc];
}
@end

@interface DefaultsObserver : NSObject {
@public unsigned changes;
}
@end
@implementation DefaultsObserver
- (void)observeValueForKeyPath:(NSString *)keyPath ofObject:(id)object
                       change:(NSDictionary *)change context:(void *)context {
    changes++;
}
@end

int main(void) {
    NSAutoreleasePool *pool = [NSAutoreleasePool new];

    NSDateFormatter *formatter = [NSDateFormatter new];
    formatter.locale = [[[NSLocale alloc] initWithLocaleIdentifier:@"en_US_POSIX"] autorelease];
    formatter.timeZone = [NSTimeZone timeZoneForSecondsFromGMT:-19800];
    formatter.dateFormat = @"yyyy-MM-dd HH:mm:ss";
    NSDate *parsed = [formatter dateFromString:@"2009-07-04 15:05:09"];
    assert(parsed != nil);
    assert([[formatter stringFromDate:parsed] isEqual:@"2009-07-04 15:05:09"]);
    assert([formatter dateFromString:@"2009-02-29 15:05:09"] == nil);
    [formatter release];

    NSString *uniqueFirst = [[NSProcessInfo processInfo] globallyUniqueString];
    NSString *uniqueSecond = [[NSProcessInfo processInfo] globallyUniqueString];
    assert(uniqueFirst.length > 0 && uniqueSecond.length > 0);
    assert(![uniqueFirst isEqual:uniqueSecond]);

    NSString *suite = [@"org.touchhle.foundation-fixture."
        stringByAppendingString:[[NSUUID UUID] UUIDString]];
    NSUserDefaults *defaults = [[NSUserDefaults alloc] initWithSuiteName:suite];
    [defaults registerDefaults:@{@"setting": @"fallback"}];
    [defaults setObject:@"saved" forKey:@"setting"];
    [defaults setObject:@"preserve" forKey:@"unrelated"];
    DefaultsObserver *observer = [DefaultsObserver new];
    [defaults addObserver:observer forKeyPath:@"setting" options:0 context:NULL];
    [defaults setObject:nil forKey:@"setting"];
    assert(observer->changes == 1);
    assert([[defaults objectForKey:@"setting"] isEqual:@"fallback"]);
    [defaults setObject:nil forKey:@"absent"];
    assert([defaults objectForKey:@"absent"] == nil);
    assert([[defaults objectForKey:@"unrelated"] isEqual:@"preserve"]);
    [defaults synchronize];
    NSDictionary *persisted = [defaults persistentDomainForName:suite];
    assert([persisted objectForKey:@"setting"] == nil);
    assert([[persisted objectForKey:@"unrelated"] isEqual:@"preserve"]);
    [defaults removeObserver:observer forKeyPath:@"setting"];
    [observer release];
    [defaults removePersistentDomainForName:suite];
    [defaults release];

    BOOL rejected = NO;
    @try {
        [[NSMutableDictionary dictionary] setObject:nil forKey:@"invalid"];
    } @catch (NSException *exception) {
        rejected = [exception.name isEqual:NSInvalidArgumentException];
    }
    assert(rejected);

    NSMutableDictionary *kvcTarget =
        [NSMutableDictionary dictionaryWithObject:@"kept" forKey:@"value"];
    [kvcTarget setValuesForKeysWithDictionary:nil];
    assert([[kvcTarget objectForKey:@"value"] isEqual:@"kept"]);

    id dictionaryClass = [NSMutableDictionary class];
    assert([dictionaryClass copyWithZone:NULL] == dictionaryClass);

    NSString *plistError = @"sentinel";
    NSData *emptyDictionaryData = [NSPropertyListSerialization
        dataFromPropertyList:[NSDictionary dictionary]
        format:NSPropertyListBinaryFormat_v1_0
        errorDescription:&plistError];
    assert(emptyDictionaryData != nil && plistError == nil);

    NSScanner *floatScanner = [NSScanner scannerWithString:@"  -1.25e+2tail"];
    float scannedFloat = 0.0f;
    assert([floatScanner scanFloat:&scannedFloat]);
    assert(scannedFloat == -125.0f && floatScanner.scanLocation == 10);
    scannedFloat = 77.0f;
    assert(![floatScanner scanFloat:&scannedFloat]);
    assert(scannedFloat == 77.0f && floatScanner.scanLocation == 10);

    NSMutableString *replaceTarget = [@"🚀cat-CAT-cat" mutableCopy];
    NSUInteger replacementCount = [replaceTarget
        replaceOccurrencesOfString:@"cat"
        withString:@"dog"
        options:NSCaseInsensitiveSearch
        range:NSMakeRange(2, 11)];
    assert(replacementCount == 3);
    assert([replaceTarget isEqual:@"🚀dog-dog-dog"]);
    [replaceTarget release];

    NSString *atomicPath = [NSTemporaryDirectory() stringByAppendingPathComponent:
        [@"touchhle-nsdata-" stringByAppendingString:[[NSUUID UUID] UUIDString]]];
    NSData *atomicPayload = [@"toybot-save" dataUsingEncoding:NSUTF8StringEncoding];
    NSError *writeError = [NSError errorWithDomain:@"sentinel" code:1 userInfo:nil];
    NSError *sentinelWriteError = writeError;
    assert([atomicPayload writeToFile:atomicPath options:NSDataWritingAtomic error:&writeError]);
    assert(writeError == sentinelWriteError);
    assert([[NSData dataWithContentsOfFile:atomicPath] isEqualToData:atomicPayload]);
    [[NSFileManager defaultManager] removeItemAtPath:atomicPath error:NULL];

    NSObject *arrayElement = [NSObject new];
    NSMutableArray *mutableArray = [NSMutableArray arrayWithObject:arrayElement];
    NSArray *immutableArray = [NSArray arrayWithObject:arrayElement];
    assert([mutableArray isKindOfClass:[NSMutableArray class]]);
    assert(![immutableArray isKindOfClass:[NSMutableArray class]]);
    [mutableArray addObject:arrayElement];
    assert(mutableArray.count == 2 && immutableArray.count == 1);
    assert([mutableArray objectAtIndex:1] == arrayElement);
    [arrayElement release];
    for (Class arrayClass in @[[NSArray class], [NSMutableArray class]]) {
        rejected = NO;
        @try {
            [arrayClass arrayWithObject:nil];
        } @catch (NSException *exception) {
            rejected = [exception.name isEqual:NSInvalidArgumentException];
        }
        assert(rejected);
    }

    CFMutableArrayRef fixedArray = CFArrayCreateMutable(NULL, 2, NULL);
    CFArrayAppendValue(fixedArray, (const void *)1);
    CFArrayAppendValue(fixedArray, (const void *)2);
    assert(CFArrayGetCount(fixedArray) == 2);
    CFArrayRemoveValueAtIndex(fixedArray, 0);
    CFArrayAppendValue(fixedArray, (const void *)3);
    assert(CFArrayGetCount(fixedArray) == 2);
    CFRelease(fixedArray);

    NSMutableURLRequest *headerRequest =
        [NSMutableURLRequest requestWithURL:[NSURL URLWithString:@"https://example.invalid/"]];
    NSMutableDictionary *suppliedHeaders =
        [NSMutableDictionary dictionaryWithObject:@"one" forKey:@"X-First"];
    headerRequest.allHTTPHeaderFields = suppliedHeaders;
    [suppliedHeaders setObject:@"two" forKey:@"X-Second"];
    assert(headerRequest.allHTTPHeaderFields.count == 1);
    assert([[headerRequest.allHTTPHeaderFields objectForKey:@"X-First"] isEqual:@"one"]);
    assert([headerRequest.allHTTPHeaderFields objectForKey:@"X-Second"] == nil);
    headerRequest.allHTTPHeaderFields = nil;
    assert(headerRequest.allHTTPHeaderFields == nil);

    NSMutableDictionary *mutableDictionary =
        [NSMutableDictionary dictionaryWithObject:@"one" forKey:@"first"];
    NSDictionary *immutableDictionary =
        [NSDictionary dictionaryWithObject:@"one" forKey:@"first"];
    assert([mutableDictionary isKindOfClass:[NSMutableDictionary class]]);
    assert(![immutableDictionary isKindOfClass:[NSMutableDictionary class]]);
    [mutableDictionary setObject:@"two" forKey:@"second"];
    assert(mutableDictionary.count == 2 && immutableDictionary.count == 1);
    NSData *dictionaryArchive =
        [NSKeyedArchiver archivedDataWithRootObject:mutableDictionary];
    NSMutableDictionary *decodedDictionary =
        [NSKeyedUnarchiver unarchiveObjectWithData:dictionaryArchive];
    assert([decodedDictionary isKindOfClass:[NSMutableDictionary class]]);
    [decodedDictionary setObject:@"three" forKey:@"third"];
    assert(decodedDictionary.count == 3);

    NSObject *setFirst = [NSObject new];
    NSObject *setSecond = [NSObject new];
    NSMutableSet *arraySet = [NSMutableSet set];
    [arraySet addObjectsFromArray:@[setFirst, setSecond, setFirst]];
    assert(arraySet.count == 2);
    assert([arraySet containsObject:setFirst]);
    assert([arraySet containsObject:setSecond]);
    [setFirst release];
    [setSecond release];

    CopyToken *element = [CopyToken new];
    NSMutableSet *source = [[NSMutableSet alloc] initWithObjects:element, nil];
    NSMutableSet *mutable = [source mutableCopy];
    NSSet *immutable = [source copy];
    NSMutableSet *another = [immutable mutableCopy];
    assert([mutable isKindOfClass:[NSMutableSet class]]);
    assert(![immutable isKindOfClass:[NSMutableSet class]]);
    assert(mutable != source && another != immutable);
    assert([immutable anyObject] == element);
    [source removeAllObjects];
    assert(mutable.count == 1 && immutable.count == 1);
    [mutable removeObject:element];
    assert(immutable.count == 1);
    [source release];
    [mutable release];
    [another release];
    [element release];
    [pool drain];
    assert(liveTokens == 1 && [immutable anyObject] == element);
    [immutable release];
    assert(liveTokens == 0);
    puts("Foundation date, process-info, defaults, string/data, mutable-collection and copy contracts passed.");
    return 0;
}
