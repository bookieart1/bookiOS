#ifndef UI_BUTTON_CONTENT_EDGE_INSETS_UIKIT_H
#define UI_BUTTON_CONTENT_EDGE_INSETS_UIKIT_H

typedef signed char BOOL;
#define nil ((id)0)

typedef float CGFloat;

typedef struct UIEdgeInsets {
  CGFloat top;
  CGFloat left;
  CGFloat bottom;
  CGFloat right;
} UIEdgeInsets;

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

@interface UIControl : UIView
@end

@interface UIButton : UIControl
- (UIEdgeInsets)contentEdgeInsets;
- (void)setContentEdgeInsets:(UIEdgeInsets)contentEdgeInsets;
@end

#endif
