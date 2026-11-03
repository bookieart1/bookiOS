/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! OpenGL ES and EAGL.
//!
//! This module is specific to OpenGL ES's role as a part of the iPhone OS API
//! surface. See [crate::gles] for other uses and a discussion of the broader
//! topic.

mod eagl;
mod gles_guest;

use touchHLE_gl_bindings::gles11::types::GLenum;

use crate::mem::ConstPtr;

pub const DYLIB: crate::dyld::HostDylib = crate::dyld::HostDylib {
    path: "/System/Library/Frameworks/OpenGLES.framework/OpenGLES",
    aliases: &[],
    class_exports: &[eagl::CLASSES],
    constant_exports: &[eagl::CONSTANTS],
    function_exports: &[gles_guest::FUNCTIONS],
};

#[derive(Default)]
pub struct State {
    /// Current EAGLContext for each thread
    current_ctxs: std::collections::HashMap<crate::ThreadId, Option<crate::objc::id>>,
    strings_cache: std::collections::HashMap<GLenum, ConstPtr<u8>>,
    /// At most 24 detailed GL snapshots are emitted after user input during
    /// one app run. This is enough to capture a few state transitions without
    /// turning normal rendering into per-frame tracing.
    input_diagnostic_remaining: u8,
    input_diagnostic_emitted: u8,
    /// A second, independently bounded budget samples guest draw-call state.
    /// It is intentionally separate from frame-presentation snapshots so a
    /// corrupted frame can be correlated with the draws that produced it.
    input_draw_diagnostic_remaining: u8,
    input_draw_diagnostic_emitted: u8,
}
impl State {
    fn current_ctx_for_thread(&mut self, thread: crate::ThreadId) -> &mut Option<crate::objc::id> {
        self.current_ctxs.entry(thread).or_insert(None);
        self.current_ctxs.get_mut(&thread).unwrap()
    }

    pub(crate) fn request_input_diagnostic_capture(&mut self) {
        const TOTAL_BUDGET: u8 = 24;
        const FRAMES_PER_INPUT: u8 = 8;
        let available = TOTAL_BUDGET.saturating_sub(self.input_diagnostic_emitted);
        self.input_diagnostic_remaining = self
            .input_diagnostic_remaining
            .max(FRAMES_PER_INPUT.min(available));

        const DRAW_TOTAL_BUDGET: u8 = 32;
        const DRAWS_PER_INPUT: u8 = 8;
        let draw_available =
            DRAW_TOTAL_BUDGET.saturating_sub(self.input_draw_diagnostic_emitted);
        self.input_draw_diagnostic_remaining = self
            .input_draw_diagnostic_remaining
            .max(DRAWS_PER_INPUT.min(draw_available));
    }

    fn take_input_diagnostic_snapshot(&mut self) -> Option<u8> {
        if self.input_diagnostic_remaining == 0 || self.input_diagnostic_emitted >= 24 {
            return None;
        }
        self.input_diagnostic_remaining -= 1;
        self.input_diagnostic_emitted += 1;
        Some(self.input_diagnostic_emitted)
    }

    fn take_input_draw_diagnostic(&mut self) -> Option<u8> {
        if self.input_draw_diagnostic_remaining == 0
            || self.input_draw_diagnostic_emitted >= 32
        {
            return None;
        }
        self.input_draw_diagnostic_remaining -= 1;
        self.input_draw_diagnostic_emitted += 1;
        Some(self.input_draw_diagnostic_emitted)
    }
}

fn sync_context<'objc, 'win: 'objc>(
    state: &mut State,
    objc: &'objc mut crate::objc::ObjC,
    window: &'win mut crate::window::Window,
    current_thread: crate::ThreadId,
) -> Box<dyn crate::gles::GLES + 'objc> {
    let gles_ctx = get_thread_context(state, objc, current_thread);
    gles_ctx.make_current(window)
}

fn get_thread_context<'objc>(
    state: &mut State,
    objc: &'objc mut crate::objc::ObjC,
    current_thread: crate::ThreadId,
) -> &'objc mut dyn crate::gles::GLESContext {
    let current_ctx = state.current_ctx_for_thread(current_thread);
    let host_obj = objc.borrow_mut::<eagl::EAGLContextHostObject>(current_ctx.unwrap());
    let gles_ctx = host_obj.gles_ctx.as_deref_mut().unwrap();
    gles_ctx
}
