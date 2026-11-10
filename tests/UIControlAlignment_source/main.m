/*
 * Focused iPhone OS guest contract for UIControl horizontal alignment.
 */

#import <UIKit/UIKit.h>

#include <stdio.h>

@interface OFGreenBorderedButton : UIButton
@end

@implementation OFGreenBorderedButton
@end

int main(void) {
  NSAutoreleasePool *pool = [NSAutoreleasePool new];
  OFGreenBorderedButton *button = [OFGreenBorderedButton new];

  if (![button isKindOfClass:[UIButton class]]) {
    fprintf(stderr, "OFGREEN_ALIGNMENT ancestry failure\n");
    return 1;
  }
  if (![button respondsToSelector:@selector(setContentHorizontalAlignment:)] ||
      ![button respondsToSelector:@selector(contentHorizontalAlignment)]) {
    fprintf(stderr, "OFGREEN_ALIGNMENT selector failure\n");
    return 2;
  }
  if ([button contentHorizontalAlignment] !=
      UIControlContentHorizontalAlignmentCenter) {
    fprintf(stderr, "OFGREEN_ALIGNMENT default failure actual=%ld\n",
            (long)[button contentHorizontalAlignment]);
    return 3;
  }

  UIControlContentHorizontalAlignment values[] = {
      UIControlContentHorizontalAlignmentLeft,
      UIControlContentHorizontalAlignmentRight,
      UIControlContentHorizontalAlignmentFill,
  };
  for (unsigned index = 0; index < sizeof(values) / sizeof(values[0]); index++) {
    [button setContentHorizontalAlignment:values[index]];
    if ([button contentHorizontalAlignment] != values[index]) {
      fprintf(stderr,
              "OFGREEN_ALIGNMENT storage failure expected=%ld actual=%ld\n",
              (long)values[index], (long)[button contentHorizontalAlignment]);
      return 4;
    }
  }

  [button release];
  [pool drain];
  printf("OFGREEN_ALIGNMENT ancestry=UIButton default=0 values=1,2,3 "
         "release=1 result=PASS\n");
  fflush(stdout);
  return 0;
}
