/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Temporary Development 2.56 iconv call-surface discriminator.

use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, MutVoidPtr};
use crate::Environment;

fn iconv_open(env: &mut Environment, to_code: ConstPtr<u8>, from_code: ConstPtr<u8>) -> MutVoidPtr {
    let to_code = env.mem.cstr_at_utf8(to_code).unwrap_or("<invalid UTF-8>");
    let from_code = env.mem.cstr_at_utf8(from_code).unwrap_or("<invalid UTF-8>");
    panic!(
        "Development 2.56 iconv preflight: from={from_code:?} to={to_code:?}"
    );
}

pub const FUNCTIONS: FunctionExports = &[export_c_func!(iconv_open(_, _))];
