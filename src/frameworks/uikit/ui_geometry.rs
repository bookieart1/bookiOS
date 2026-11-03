//! `UIGeometry.h`
//!
//! See also [crate::frameworks::core_graphics::cg_geometry].

use crate::abi::{impl_GuestRet_for_large_struct, GuestArg};
use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::ns_string;
use crate::mem::SafeRead;
use crate::objc::{autorelease, id};
use crate::Environment;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(C, packed)]
pub struct UIEdgeInsets {
    pub top: CGFloat,
    pub left: CGFloat,
    pub bottom: CGFloat,
    pub right: CGFloat,
}
unsafe impl SafeRead for UIEdgeInsets {}
impl_GuestRet_for_large_struct!(UIEdgeInsets);
impl GuestArg for UIEdgeInsets {
    const REG_COUNT: usize = 4;

    fn from_regs(regs: &[u32]) -> Self {
        UIEdgeInsets {
            top: GuestArg::from_regs(&regs[0..1]),
            left: GuestArg::from_regs(&regs[1..2]),
            bottom: GuestArg::from_regs(&regs[2..3]),
            right: GuestArg::from_regs(&regs[3..4]),
        }
    }
    fn to_regs(self, regs: &mut [u32]) {
        self.top.to_regs(&mut regs[0..1]);
        self.left.to_regs(&mut regs[1..2]);
        self.bottom.to_regs(&mut regs[2..3]);
        self.right.to_regs(&mut regs[3..4]);
    }
}

// Apple's documentation says all of these return zeroes if the input is not
// well-formed.
pub fn CGPointFromString(env: &mut Environment, string: id) -> CGPoint {
    // TODO: avoid copy
    ns_string::to_rust_string(env, string)
        .parse()
        .unwrap_or_default()
}
pub fn CGSizeFromString(env: &mut Environment, string: id) -> CGSize {
    // TODO: avoid copy
    ns_string::to_rust_string(env, string)
        .parse()
        .unwrap_or_default()
}
pub fn CGRectFromString(env: &mut Environment, string: id) -> CGRect {
    // TODO: avoid copy
    ns_string::to_rust_string(env, string)
        .parse()
        .unwrap_or_default()
}

fn parse_edge_insets(string: &str) -> Option<UIEdgeInsets> {
    let contents = string.trim().strip_prefix('{')?.strip_suffix('}')?;
    let mut components = contents.split(',').map(str::trim);
    let insets = UIEdgeInsets {
        top: components.next()?.parse().ok()?,
        left: components.next()?.parse().ok()?,
        bottom: components.next()?.parse().ok()?,
        right: components.next()?.parse().ok()?,
    };
    if components.next().is_some() {
        return None;
    }
    Some(insets)
}

pub fn UIEdgeInsetsFromString(env: &mut Environment, string: id) -> UIEdgeInsets {
    if string.is_null() {
        return UIEdgeInsets::default();
    }
    parse_edge_insets(&ns_string::to_rust_string(env, string)).unwrap_or_default()
}

pub fn NSStringFromCGPoint(env: &mut Environment, point: CGPoint) -> id {
    let s = ns_string::from_rust_string(env, point.to_string());
    autorelease(env, s)
}
pub fn NSStringFromCGSize(env: &mut Environment, size: CGSize) -> id {
    let s = ns_string::from_rust_string(env, size.to_string());
    autorelease(env, s)
}
pub fn NSStringFromCGRect(env: &mut Environment, rect: CGRect) -> id {
    let s = ns_string::from_rust_string(env, rect.to_string());
    autorelease(env, s)
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CGPointFromString(_)),
    export_c_func!(CGSizeFromString(_)),
    export_c_func!(CGRectFromString(_)),
    export_c_func!(UIEdgeInsetsFromString(_)),
    export_c_func!(NSStringFromCGPoint(_)),
    export_c_func!(NSStringFromCGSize(_)),
    export_c_func!(NSStringFromCGRect(_)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::GuestRet;

    #[test]
    fn edge_insets_parser_accepts_historical_four_component_format() {
        let UIEdgeInsets {
            top,
            left,
            bottom,
            right,
        } = parse_edge_insets("{1.5, -2, 3.25, 4}").unwrap();
        assert_eq!((top, left, bottom, right), (1.5, -2.0, 3.25, 4.0));
    }

    #[test]
    fn malformed_edge_insets_return_zero_and_use_struct_return_abi() {
        assert!(parse_edge_insets("{1, 2, 3}").is_none());
        assert!(parse_edge_insets("{1, 2, 3, 4, 5}").is_none());
        assert!(parse_edge_insets("1, 2, 3, 4").is_none());
        assert_eq!(
            <UIEdgeInsets as GuestRet>::SIZE_IN_MEM,
            Some(crate::mem::guest_size_of::<UIEdgeInsets>())
        );
    }
}

pub const CONSTANTS: ConstantExports = &[(
    "_UIEdgeInsetsZero",
    HostConstant::Custom(|env| {
        env.mem
            .alloc_and_write(UIEdgeInsets::default())
            .cast()
            .cast_const()
    }),
)];
