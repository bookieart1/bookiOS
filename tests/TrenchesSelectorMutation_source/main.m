/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#import <Foundation/Foundation.h>

#include <stdio.h>
#include <string.h>

typedef enum {
  TRActionNone = 0,
  TRActionRemoveSelf = 1,
  TRActionRemoveTarget = 2,
} TRAction;

static char callback_order[8];
static unsigned callback_count;
static int callback_overflow;

@interface TRCallback : NSObject {
  char identifier;
  NSMutableArray *receiver;
  TRCallback *target;
  TRAction action;
}
- (id)initWithIdentifier:(char)newIdentifier;
- (void)configureReceiver:(NSMutableArray *)newReceiver
                   action:(TRAction)newAction
                   target:(TRCallback *)newTarget;
- (char)identifier;
- (void)fire;
@end

@implementation TRCallback
- (id)initWithIdentifier:(char)newIdentifier {
  self = [super init];
  if (self != nil) {
    identifier = newIdentifier;
    receiver = nil;
    target = nil;
    action = TRActionNone;
  }
  return self;
}

- (void)configureReceiver:(NSMutableArray *)newReceiver
                   action:(TRAction)newAction
                   target:(TRCallback *)newTarget {
  // Deliberately non-owning. The case owns the receiver and A-D independently.
  receiver = newReceiver;
  action = newAction;
  target = newTarget;
}

- (char)identifier {
  return identifier;
}

- (void)fire {
  if (callback_count < sizeof(callback_order) - 1) {
    callback_order[callback_count++] = identifier;
    callback_order[callback_count] = '\0';
  } else {
    callback_overflow = 1;
  }

  if (action == TRActionRemoveSelf) {
    [receiver removeObjectIdenticalTo:self];
  } else if (action == TRActionRemoveTarget) {
    [receiver removeObjectIdenticalTo:target];
  }
}
@end

static int run_case(const char *case_name) {
  NSAutoreleasePool *pool = [NSAutoreleasePool new];
  TRCallback *a = [[TRCallback alloc] initWithIdentifier:'A'];
  TRCallback *b = [[TRCallback alloc] initWithIdentifier:'B'];
  TRCallback *c = [[TRCallback alloc] initWithIdentifier:'C'];
  TRCallback *d = [[TRCallback alloc] initWithIdentifier:'D'];
  NSMutableArray *array =
      [[NSMutableArray alloc] initWithObjects:a, b, c, d, nil];
  const char *expected_membership = NULL;

  memset(callback_order, 0, sizeof(callback_order));
  callback_count = 0;
  callback_overflow = 0;

  [a configureReceiver:array action:TRActionNone target:nil];
  [b configureReceiver:array action:TRActionNone target:nil];
  [c configureReceiver:array action:TRActionNone target:nil];
  [d configureReceiver:array action:TRActionNone target:nil];

  if (!strcmp(case_name, "control")) {
    expected_membership = "ABCD";
  } else if (!strcmp(case_name, "self-remove")) {
    [b configureReceiver:array action:TRActionRemoveSelf target:nil];
    [d configureReceiver:array action:TRActionRemoveSelf target:nil];
    expected_membership = "AC";
  } else if (!strcmp(case_name, "remove-later")) {
    [b configureReceiver:array action:TRActionRemoveTarget target:d];
    expected_membership = "ABC";
  } else {
    fprintf(stderr, "TRENCHESTEST invalid-case=%s\n", case_name);
    return 64;
  }

  [array makeObjectsPerformSelector:@selector(fire)];

  char membership[8] = {0};
  NSUInteger count = [array count];
  if (count >= sizeof(membership)) {
    fprintf(stderr, "TRENCHESTEST membership-overflow case=%s count=%lu\n",
            case_name, (unsigned long)count);
    return 65;
  }
  for (NSUInteger index = 0; index < count; index++) {
    membership[index] = [(TRCallback *)[array objectAtIndex:index] identifier];
  }

  int result = 0;
  if (callback_overflow || strcmp(callback_order, "ABCD") ||
      strcmp(membership, expected_membership)) {
    result = 1;
  }
  printf("TRENCHESTEST case=%s callbacks=%s membership=%s returned=1 "
         "overflow=%d result=%s\n",
         case_name, callback_order, membership, callback_overflow,
         result == 0 ? "PASS" : "FAIL");
  fflush(stdout);

  [array release];
  [a release];
  [b release];
  [c release];
  [d release];
  [pool drain];
  return result;
}

int main(int argc, char **argv) {
  if (argc != 2) {
    fprintf(stderr, "Usage: TrenchesSelectorMutation <case>\n");
    return 64;
  }
  return run_case(argv[1]);
}
