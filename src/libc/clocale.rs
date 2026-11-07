/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `clocale.h`

use std::collections::hash_map::Entry;

use crate::dyld::FunctionExports;
use crate::environment::Environment;
use crate::export_c_func;
use crate::mem::{ConstPtr, MutPtr, MutVoidPtr, Ptr, SafeRead};

pub type LocaleCategory = i32;
pub const LC_ALL: LocaleCategory = 0;
pub const LC_COLLATE: LocaleCategory = 1;
pub const LC_CTYPE: LocaleCategory = 2;
pub const LC_MONETARY: LocaleCategory = 3;
pub const LC_NUMERIC: LocaleCategory = 4;
pub const LC_TIME: LocaleCategory = 5;
pub const LC_MESSAGES: LocaleCategory = 6;
const LC_GLOBAL_LOCALE: MutVoidPtr = Ptr::from_bits(u32::MAX);

#[derive(Default)]
pub struct State {
    locale: std::collections::HashMap<LocaleCategory, MutPtr<u8>>,
    thread_locale: std::collections::HashMap<usize, MutVoidPtr>,
    langinfo: std::collections::HashMap<i32, MutPtr<u8>>,
    lconv: Option<MutPtr<LConv>>,
}

/// Darwin's 32-bit guest ABI layout. This must not use the host platform's
/// libc structure because both pointer width and locale state differ.
#[derive(Copy, Clone)]
#[repr(C)]
struct LConv {
    decimal_point: MutPtr<u8>,
    thousands_sep: MutPtr<u8>,
    grouping: MutPtr<u8>,
    int_curr_symbol: MutPtr<u8>,
    currency_symbol: MutPtr<u8>,
    mon_decimal_point: MutPtr<u8>,
    mon_thousands_sep: MutPtr<u8>,
    mon_grouping: MutPtr<u8>,
    positive_sign: MutPtr<u8>,
    negative_sign: MutPtr<u8>,
    int_frac_digits: i8,
    frac_digits: i8,
    p_cs_precedes: i8,
    p_sep_by_space: i8,
    n_cs_precedes: i8,
    n_sep_by_space: i8,
    p_sign_posn: i8,
    n_sign_posn: i8,
    int_p_cs_precedes: i8,
    int_n_cs_precedes: i8,
    int_p_sep_by_space: i8,
    int_n_sep_by_space: i8,
    int_p_sign_posn: i8,
    int_n_sign_posn: i8,
}
unsafe impl SafeRead for LConv {}

pub fn setlocale(
    env: &mut Environment,
    category: LocaleCategory,
    locale: ConstPtr<u8>,
) -> MutPtr<u8> {
    assert!(matches!(
        category,
        LC_ALL | LC_COLLATE | LC_CTYPE | LC_MONETARY | LC_NUMERIC | LC_TIME | LC_MESSAGES
    ));
    if !locale.is_null() {
        // TODO: Handle empty locale string and ensure the combination of
        // category and locale is valid.
        let locale_cstr = env.mem.cstr_at(locale).to_owned();
        assert_ne!(locale_cstr.len(), 0);
        let new_locale = env.mem.alloc_and_write_cstr(locale_cstr.as_slice());
        if let Some(old_locale) = env.libc_state.clocale.locale.insert(category, new_locale) {
            env.mem.free(old_locale.cast())
        };
    } else if let Entry::Vacant(entry) = env.libc_state.clocale.locale.entry(category) {
        let default_locale = env.mem.alloc_and_write_cstr(b"C");
        entry.insert(default_locale);
    }
    env.libc_state.clocale.locale.get(&category).unwrap().cast()
}

fn newlocale(
    env: &mut Environment,
    _category_mask: i32,
    locale: ConstPtr<u8>,
    _base: MutVoidPtr,
) -> MutVoidPtr {
    let locale_name = if locale.is_null() {
        b"C".to_vec()
    } else {
        let requested = env.mem.cstr_at(locale);
        if requested.is_empty() {
            b"C".to_vec()
        } else {
            requested.to_owned()
        }
    };
    env.mem.alloc_and_write_cstr(&locale_name).cast()
}

fn duplocale(env: &mut Environment, locale: MutVoidPtr) -> MutVoidPtr {
    let locale_name = if locale == LC_GLOBAL_LOCALE || locale.is_null() {
        b"C".to_vec()
    } else {
        env.mem.cstr_at(locale.cast_const().cast()).to_owned()
    };
    env.mem.alloc_and_write_cstr(&locale_name).cast()
}

fn freelocale(env: &mut Environment, locale: MutVoidPtr) {
    if !locale.is_null() && locale != LC_GLOBAL_LOCALE {
        env.mem.free(locale);
    }
}

fn uselocale(env: &mut Environment, locale: MutVoidPtr) -> MutVoidPtr {
    let current = env
        .libc_state
        .clocale
        .thread_locale
        .get(&env.current_thread)
        .copied()
        .unwrap_or(LC_GLOBAL_LOCALE);
    if !locale.is_null() {
        env.libc_state
            .clocale
            .thread_locale
            .insert(env.current_thread, locale);
    }
    current
}

fn nl_langinfo(env: &mut Environment, item: i32) -> MutPtr<u8> {
    if let Some(&cached) = env.libc_state.clocale.langinfo.get(&item) {
        return cached;
    }
    // Darwin's C/POSIX-locale values and numeric constants from _langinfo.h.
    // Locale selection is not yet modelled beyond C, so these deterministic
    // fallbacks are more accurate than guessing from the host locale.
    const DAYS: [&[u8]; 7] = [b"Sunday", b"Monday", b"Tuesday", b"Wednesday", b"Thursday", b"Friday", b"Saturday"];
    const ABDAYS: [&[u8]; 7] = [b"Sun", b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat"];
    const MONTHS: [&[u8]; 12] = [b"January", b"February", b"March", b"April", b"May", b"June", b"July", b"August", b"September", b"October", b"November", b"December"];
    const ABMONTHS: [&[u8]; 12] = [b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec"];
    let value: &[u8] = match item {
        0 => b"US-ASCII",
        1 => b"%a %b %e %H:%M:%S %Y",
        2 => b"%m/%d/%y",
        3 => b"%H:%M:%S",
        4 => b"%I:%M:%S %p",
        5 => b"AM",
        6 => b"PM",
        7..=13 => DAYS[(item - 7) as usize],
        14..=20 => ABDAYS[(item - 14) as usize],
        21..=32 => MONTHS[(item - 21) as usize],
        33..=44 => ABMONTHS[(item - 33) as usize],
        50 => b".",
        52 => b"^[yY]",
        53 => b"^[nN]",
        54 => b"yes",
        55 => b"no",
        57 => b"md",
        _ => b"",
    };
    let guest = env.mem.alloc_and_write_cstr(value);
    env.libc_state.clocale.langinfo.insert(item, guest);
    guest
}

fn localeconv(env: &mut Environment) -> MutPtr<LConv> {
    if let Some(lconv) = env.libc_state.clocale.lconv {
        return lconv;
    }
    let decimal_point = env.mem.alloc_and_write_cstr(b".");
    let empty = env.mem.alloc_and_write_cstr(b"");
    let unavailable = i8::MAX;
    let lconv = env.mem.alloc_and_write(LConv {
        decimal_point,
        thousands_sep: empty,
        grouping: empty,
        int_curr_symbol: empty,
        currency_symbol: empty,
        mon_decimal_point: empty,
        mon_thousands_sep: empty,
        mon_grouping: empty,
        positive_sign: empty,
        negative_sign: empty,
        int_frac_digits: unavailable,
        frac_digits: unavailable,
        p_cs_precedes: unavailable,
        p_sep_by_space: unavailable,
        n_cs_precedes: unavailable,
        n_sep_by_space: unavailable,
        p_sign_posn: unavailable,
        n_sign_posn: unavailable,
        int_p_cs_precedes: unavailable,
        int_n_cs_precedes: unavailable,
        int_p_sep_by_space: unavailable,
        int_n_sep_by_space: unavailable,
        int_p_sign_posn: unavailable,
        int_n_sign_posn: unavailable,
    });
    env.libc_state.clocale.lconv = Some(lconv);
    lconv
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(setlocale(_, _)),
    export_c_func!(newlocale(_, _, _)),
    export_c_func!(duplocale(_)),
    export_c_func!(freelocale(_)),
    export_c_func!(uselocale(_)),
    export_c_func!(nl_langinfo(_)),
    export_c_func!(localeconv()),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_locale_langinfo_is_stable_and_uses_darwin_item_numbers() {
        let mut env = Environment::new_fake_for_test();
        let codeset = nl_langinfo(&mut env, 0);
        assert_eq!(env.mem.cstr_at(codeset.cast_const()), b"US-ASCII");
        assert_eq!(nl_langinfo(&mut env, 0), codeset);
        let sunday = nl_langinfo(&mut env, 7);
        assert_eq!(env.mem.cstr_at(sunday.cast_const()), b"Sunday");
        let invalid = nl_langinfo(&mut env, 999);
        assert_eq!(env.mem.cstr_at(invalid.cast_const()), b"");
    }

    #[test]
    fn dev_244_localeconv_returns_stable_guest_c_locale_storage() {
        let mut env = Environment::new_fake_for_test();
        let first = localeconv(&mut env);
        let second = localeconv(&mut env);
        assert_eq!(first, second);
        let value = env.mem.read(first.cast_const());
        assert_eq!(env.mem.cstr_at(value.decimal_point.cast_const()), b".");
        assert_eq!(env.mem.cstr_at(value.thousands_sep.cast_const()), b"");
        assert_eq!(value.frac_digits, i8::MAX);
        assert_eq!(std::mem::size_of::<LConv>(), 56);
    }
}
