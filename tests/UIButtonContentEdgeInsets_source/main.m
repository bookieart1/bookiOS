/* Focused iPhone OS guest contract for UIButton contentEdgeInsets. */

#import <UIKit/UIKit.h>

#include <stdio.h>

@interface OFDefaultButton : UIButton
@end
@implementation OFDefaultButton
@end

@interface OFGreenBorderedButton : OFDefaultButton
@end
@implementation OFGreenBorderedButton
@end

static BOOL equalInsets(UIEdgeInsets lhs, UIEdgeInsets rhs) {
  return lhs.top == rhs.top && lhs.left == rhs.left &&
         lhs.bottom == rhs.bottom && lhs.right == rhs.right;
}

int main(void) {
  NSAutoreleasePool *pool = [NSAutoreleasePool new];
  OFGreenBorderedButton *guestButton = [OFGreenBorderedButton new];
  UIButton *ownerButton = [UIButton new];
  UIControl *unrelatedControl = [UIControl new];
  UIEdgeInsets zero = {0.0f, 0.0f, 0.0f, 0.0f};
  UIEdgeInsets value = {1.25f, -2.5f, 3.75f, 4.5f};

  if (![guestButton isKindOfClass:[UIButton class]]) {
    fprintf(stderr, "CONTENT_EDGE_INSETS ancestry failure\n");
    return 1;
  }
  if (![guestButton respondsToSelector:@selector(setContentEdgeInsets:)] ||
      ![guestButton respondsToSelector:@selector(contentEdgeInsets)]) {
    fprintf(stderr, "CONTENT_EDGE_INSETS inherited selector failure\n");
    return 2;
  }
  if ([unrelatedControl respondsToSelector:@selector(setContentEdgeInsets:)] ||
      [unrelatedControl respondsToSelector:@selector(contentEdgeInsets)]) {
    fprintf(stderr, "CONTENT_EDGE_INSETS owner boundary failure\n");
    return 3;
  }
  if (!equalInsets([guestButton contentEdgeInsets], zero) ||
      !equalInsets([ownerButton contentEdgeInsets], zero)) {
    fprintf(stderr, "CONTENT_EDGE_INSETS default failure\n");
    return 4;
  }

  [guestButton setContentEdgeInsets:value];
  [ownerButton setContentEdgeInsets:value];
  if (!equalInsets([guestButton contentEdgeInsets], value) ||
      !equalInsets([ownerButton contentEdgeInsets], value)) {
    fprintf(stderr, "CONTENT_EDGE_INSETS ABI/storage failure\n");
    return 5;
  }

  [unrelatedControl release];
  [ownerButton release];
  [guestButton release];
  [pool drain];
  printf("CONTENT_EDGE_INSETS ancestry=UIButton owner=UIButton unrelated=UIControl "
         "default=0,0,0,0 value=1.25,-2.5,3.75,4.5 release=1 result=PASS\n");
  fflush(stdout);
  return 0;
}
