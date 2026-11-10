#ifndef UI_CONTROL_ALIGNMENT_UIKIT_H
#define UI_CONTROL_ALIGNMENT_UIKIT_H

typedef signed char BOOL;
#define nil ((id)0)

typedef long NSInteger;

@interface NSObject {
  Class isa;
}
+ (id)alloc;
+ (id)new;
+ (Class)class;
- (id)init;
- (void)release;
- (BOOL)isKindOfClass:(Class)classObject;
- (BOOL)respondsToSelector:(SEL)selector;
@end

@interface NSAutoreleasePool : NSObject
- (void)drain;
@end

@interface UIView : NSObject
@end

typedef enum {
  UIControlContentHorizontalAlignmentCenter = 0,
  UIControlContentHorizontalAlignmentLeft = 1,
  UIControlContentHorizontalAlignmentRight = 2,
  UIControlContentHorizontalAlignmentFill = 3,
} UIControlContentHorizontalAlignment;

@interface UIControl : UIView
- (UIControlContentHorizontalAlignment)contentHorizontalAlignment;
- (void)setContentHorizontalAlignment:
    (UIControlContentHorizontalAlignment)contentHorizontalAlignment;
@end

@interface UIButton : UIControl
@end

#endif
