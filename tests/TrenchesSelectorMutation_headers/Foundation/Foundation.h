/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Declaration-only fixture input derived from the project-owned Phase 8A
 * LegacyInterfaces.h, SHA-256
 * 385b48d35b620bad48939b1727ae59f15fa9c73ba845a9e6d855a89fb8db76f1.
 */

#ifndef TRENCHES_SELECTOR_MUTATION_FOUNDATION_H
#define TRENCHES_SELECTOR_MUTATION_FOUNDATION_H

typedef signed char BOOL;
#define YES ((BOOL)1)
#define NO ((BOOL)0)
#define nil ((id)0)

typedef unsigned long NSUInteger;

@interface NSObject {
  Class isa;
}
+ (id)alloc;
+ (id)new;
- (id)init;
- (void)release;
@end

@interface NSAutoreleasePool : NSObject
- (void)drain;
@end

@interface NSArray : NSObject
- (NSUInteger)count;
- (id)objectAtIndex:(NSUInteger)index;
- (void)makeObjectsPerformSelector:(SEL)selector;
@end

@interface NSMutableArray : NSArray
- (id)initWithObjects:(id)first, ...;
- (void)removeObjectIdenticalTo:(id)object;
@end

#endif
