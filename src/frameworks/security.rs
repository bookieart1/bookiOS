/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Bounded Security framework support.
//!
//! The first implemented slice is deliberately limited to the local Internet
//! password CRUD contract used by the OpenFeint experiment. It is not a host
//! keychain bridge and does not implement certificates, trust, access groups,
//! authentication UI, or online identity.

mod sec_item;

pub const DYLIB: crate::dyld::HostDylib = crate::dyld::HostDylib {
    path: "/System/Library/Frameworks/Security.framework/Security",
    aliases: &[],
    class_exports: &[],
    constant_exports: &[sec_item::CONSTANTS],
    function_exports: &[sec_item::FUNCTIONS],
};
