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

const GL_ERROR_CANDIDATES: usize = 32;
const GL_VISUAL_HISTORY: usize = 64;

#[derive(Clone, Copy, Default)]
struct GlErrorCandidate {
    call: &'static str,
    args: [u32; 4],
}

#[derive(Clone, Copy, Default)]
struct GlVisualCall {
    call: &'static str,
    args: [u32; 4],
}

pub const DYLIB: crate::dyld::HostDylib = crate::dyld::HostDylib {
    path: "/System/Library/Frameworks/OpenGLES.framework/OpenGLES",
    aliases: &[],
    class_exports: &[eagl::CLASSES],
    constant_exports: &[eagl::CONSTANTS],
    function_exports: &[eagl::FUNCTIONS, gles_guest::FUNCTIONS],
};

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
    /// Fixed-size history of guest calls that can create or expose an
    /// incomplete-framebuffer error. It is only printed when an error is
    /// observed, so healthy gameplay gets no additional log traffic.
    gl_error_candidates: [GlErrorCandidate; GL_ERROR_CANDIDATES],
    gl_error_candidate_written: u64,
    /// Fixed-size visual-state history retained independently of glGetError.
    /// It is emitted only at an explicit recorder dump, so healthy gameplay
    /// does not produce per-call logging.
    gl_visual_history: [GlVisualCall; GL_VISUAL_HISTORY],
    gl_visual_history_written: u64,
    gl_visual_history_dumped: bool,
    /// Observe at most the first 512 guest draw/clear boundaries; stop on the
    /// first incomplete FBO. This never reads or consumes the GL error flag.
    framebuffer_probes: u16,
    framebuffer_fault_captured: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            current_ctxs: Default::default(),
            strings_cache: Default::default(),
            input_diagnostic_remaining: 0,
            input_diagnostic_emitted: 0,
            input_draw_diagnostic_remaining: 0,
            input_draw_diagnostic_emitted: 0,
            gl_error_candidates: [GlErrorCandidate::default(); GL_ERROR_CANDIDATES],
            gl_error_candidate_written: 0,
            gl_visual_history: [GlVisualCall::default(); GL_VISUAL_HISTORY],
            gl_visual_history_written: 0,
            gl_visual_history_dumped: false,
            framebuffer_probes: 0,
            framebuffer_fault_captured: false,
        }
    }
}
impl State {
    fn take_framebuffer_probe(&mut self) -> Option<u16> {
        if self.framebuffer_fault_captured || self.framebuffer_probes >= 512 {
            return None;
        }
        self.framebuffer_probes += 1;
        Some(self.framebuffer_probes)
    }
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

    pub(crate) fn note_gl_error_candidate(&mut self, call: &'static str, args: [u32; 4]) {
        let index = self.gl_error_candidate_written as usize % GL_ERROR_CANDIDATES;
        self.gl_error_candidates[index] = GlErrorCandidate { call, args };
        self.gl_error_candidate_written = self.gl_error_candidate_written.saturating_add(1);
    }

    pub(crate) fn note_gl_error_observation(&mut self, error: GLenum, source: &str) {
        if error == 0 {
            self.gl_error_candidate_written = 0;
            return;
        }
        self.log_gl_error_candidates(source, error);
    }

    pub(crate) fn note_visual_call(&mut self, call: &'static str, args: [u32; 4]) {
        let index = self.gl_visual_history_written as usize % GL_VISUAL_HISTORY;
        self.gl_visual_history[index] = GlVisualCall { call, args };
        self.gl_visual_history_written = self.gl_visual_history_written.saturating_add(1);
    }

    pub(crate) fn log_visual_history(&mut self, reason: &str) {
        if self.gl_visual_history_dumped {
            return;
        }
        self.gl_visual_history_dumped = true;
        let count = (self.gl_visual_history_written as usize).min(GL_VISUAL_HISTORY);
        log!(
            "OpenGL ES bounded visual history: reason={reason:?}, newest {count}, total={}",
            self.gl_visual_history_written
        );
        let start = self.gl_visual_history_written.saturating_sub(count as u64);
        for sequence in start..self.gl_visual_history_written {
            let event = self.gl_visual_history[sequence as usize % GL_VISUAL_HISTORY];
            log!(
                "OpenGL ES visual #{}: {} args={:08x?}",
                sequence + 1,
                event.call,
                event.args
            );
        }
    }

    pub(crate) fn log_gl_error_candidates(&self, source: &str, error: GLenum) {
        let count = (self.gl_error_candidate_written as usize).min(GL_ERROR_CANDIDATES);
        log!(
            "OpenGL ES bounded error provenance: source={source:?}, error={error:#x}, newest {count} candidate calls"
        );
        let start = self.gl_error_candidate_written.saturating_sub(count as u64);
        for sequence in start..self.gl_error_candidate_written {
            let candidate = self.gl_error_candidates[sequence as usize % GL_ERROR_CANDIDATES];
            log!(
                "OpenGL ES error candidate #{}: {} args={:08x?}",
                sequence + 1,
                candidate.call,
                candidate.args
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cocos_contract_framebuffer_probes_are_finite_and_freeze() {
        let mut state = State::default();
        for i in 1..=512 { assert_eq!(state.take_framebuffer_probe(), Some(i)); }
        assert_eq!(state.take_framebuffer_probe(), None);
        let mut state = State::default();
        assert_eq!(state.take_framebuffer_probe(), Some(1));
        state.framebuffer_fault_captured = true;
        assert_eq!(state.take_framebuffer_probe(), None);
    }

    #[test]
    fn gl_error_candidate_history_is_bounded_and_clears_after_a_clean_observation() {
        let mut state = State::default();
        for value in 0..(GL_ERROR_CANDIDATES as u32 + 5) {
            state.note_gl_error_candidate("glDrawArrays", [value, 0, 0, 0]);
        }
        assert_eq!(state.gl_error_candidate_written, 37);
        assert_eq!(
            state.gl_error_candidates[4].args[0],
            GL_ERROR_CANDIDATES as u32 + 4
        );
        state.note_gl_error_observation(0, "guest glGetError");
        assert_eq!(state.gl_error_candidate_written, 0);
    }

    #[test]
    fn visual_history_is_bounded_and_survives_clean_error_observations() {
        let mut state = State::default();
        for value in 0..(GL_VISUAL_HISTORY as u32 + 3) {
            state.note_visual_call("glTexImage2D", [value, 0, 0, 0]);
        }
        state.note_gl_error_observation(0, "guest glGetError");
        assert_eq!(state.gl_visual_history_written, 67);
        assert_eq!(state.gl_visual_history[2].args[0], 66);
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
