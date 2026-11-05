/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The UIKit framework.
//!
//! For the time being the focus of this project is on running games, which are
//! likely to use UIKit in very simple and limited ways, so this implementation
//! will probably take a lot of shortcuts.

use crate::{msg, Environment};
use std::time::{Duration, Instant};

pub mod ui_accelerometer;
pub mod ui_activity_indicator_view;
pub mod ui_application;
pub mod ui_bar_item;
pub mod ui_color;
pub mod ui_device;
pub mod ui_event;
pub mod ui_font;
pub mod ui_geometry;
pub mod ui_gesture_recognizer;
pub mod ui_graphics;
pub mod ui_image;
pub mod ui_image_picker_controller;
pub mod ui_local_notification;
pub mod ui_navigation_item;
pub mod ui_nib;
pub mod ui_pasteboard;
pub mod ui_responder;
pub mod ui_screen;
pub mod ui_touch;
pub mod ui_view;
pub mod ui_view_controller;

pub const DYLIB: crate::dyld::HostDylib = crate::dyld::HostDylib {
    path: "/System/Library/Frameworks/UIKit.framework/UIKit",
    aliases: &[],
    class_exports: &[
        ui_accelerometer::CLASSES,
        ui_activity_indicator_view::CLASSES,
        ui_bar_item::CLASSES,
        ui_bar_item::ui_bar_button_item::CLASSES,
        ui_application::CLASSES,
        ui_color::CLASSES,
        ui_device::CLASSES,
        ui_event::CLASSES,
        ui_font::CLASSES,
        ui_gesture_recognizer::CLASSES,
        ui_image::CLASSES,
        ui_image_picker_controller::CLASSES,
        ui_local_notification::CLASSES,
        ui_navigation_item::CLASSES,
        ui_nib::CLASSES,
        ui_pasteboard::CLASSES,
        ui_responder::CLASSES,
        ui_screen::CLASSES,
        ui_touch::CLASSES,
        ui_view::CLASSES,
        ui_view::ui_alert_view::CLASSES,
        ui_view::ui_control::CLASSES,
        ui_view::ui_control::ui_button::CLASSES,
        ui_view::ui_control::ui_segmented_control::CLASSES,
        ui_view::ui_control::ui_slider::CLASSES,
        ui_view::ui_control::ui_text_field::CLASSES,
        ui_view::ui_control::ui_switch::CLASSES,
        ui_view::ui_image_view::CLASSES,
        ui_view::ui_label::CLASSES,
        ui_view::ui_navigation_bar::CLASSES,
        ui_view::ui_page_control::CLASSES,
        ui_view::ui_picker_view::CLASSES,
        ui_view::ui_progress_view::CLASSES,
        ui_view::ui_scroll_view::CLASSES,
        ui_view::ui_scroll_view::ui_table_view::CLASSES,
        ui_view::ui_scroll_view::ui_text_view::CLASSES,
        ui_view::ui_table_view_cell::CLASSES,
        ui_view::ui_toolbar::CLASSES,
        ui_view::ui_web_view::CLASSES,
        ui_view::ui_window::CLASSES,
        ui_view_controller::CLASSES,
        ui_view_controller::ui_navigation_controller::CLASSES,
        ui_view_controller::ui_table_view_controller::CLASSES,
    ],
    constant_exports: &[
        ui_application::CONSTANTS,
        ui_device::CONSTANTS,
        ui_geometry::CONSTANTS,
        ui_local_notification::CONSTANTS,
        ui_pasteboard::CONSTANTS,
        ui_view::ui_control::ui_text_field::CONSTANTS,
        ui_view::ui_window::CONSTANTS,
    ],
    function_exports: &[
        ui_application::FUNCTIONS,
        ui_geometry::FUNCTIONS,
        ui_graphics::FUNCTIONS,
        ui_image::FUNCTIONS,
    ],
};

#[derive(Default)]
pub struct State {
    ui_accelerometer: ui_accelerometer::State,
    ui_application: ui_application::State,
    ui_color: ui_color::State,
    ui_device: ui_device::State,
    ui_font: ui_font::State,
    ui_graphics: ui_graphics::State,
    ui_image: ui_image::State,
    ui_pasteboard: ui_pasteboard::State,
    ui_screen: ui_screen::State,
    ui_touch: ui_touch::State,
    pub ui_view: ui_view::State,
    ui_responder: ui_responder::State,
}

/// For use by `NSRunLoop`: handles any events that have queued up.
///
/// Returns the next time this function must be called, if any, e.g. the next
/// time an accelerometer input is due.
pub fn handle_events(env: &mut Environment) -> Option<Instant> {
    use crate::window::Event;
    use crate::window::TextInputEvent;

    // NSRunLoop will never call this function in headless mode.
    while let Some(event) = env.window_mut().pop_event() {
        match event {
            Event::Quit => {
                echo!("User requested quit, exiting.");
                ui_application::exit(env);
            }
            Event::TouchesDown(..) | Event::TouchesMove(..) | Event::TouchesUp(..) => {
                ui_touch::handle_event(env, event)
            }
            Event::AppWillResignActive => {
                // A transient host interruption (including the screenshot
                // editor) pauses the whole guest. The host SDL loop remains
                // alive so it can observe foregrounding or real termination.
                log!("Handling app-will-resign-active event: pausing guest.");
                ui_application::notify_will_resign_active(env);
                let result =
                    env.on_parent_stack_in_coroutine(|window, _| window.wait_until_foreground());
                match result {
                    crate::window::ForegroundWaitResult::Resumed => {
                        log!("Host became active: resuming guest.");
                        ui_application::notify_did_become_active(env);
                    }
                    crate::window::ForegroundWaitResult::Terminating => {
                        log!("Host is terminating while guest is inactive.");
                        ui_application::exit(env);
                    }
                }
            }
            Event::AppWillTerminate => {
                log!("Handling app-will-terminate event.");
                ui_application::exit(env);
            }
            Event::EnterDebugger => {
                if env.is_debugging_enabled() {
                    log!("Handling EnterDebugger event: entering debugger.");
                    env.enter_debugger(/* reason: */ None);
                } else {
                    log!("Ignoring EnterDebugger event: no debugger connected.");
                }
            }
            Event::HomeButton => {
                // Pressing Home on iPhone OS 2 and 3 quits the app, which is
                // told first so it can save its progress. Only do that once,
                // even if Home is pressed again before we've returned.
                if !env.return_to_app_picker {
                    log!("Handling HomeButton event: returning to app picker.");
                    echo!(
                        "iDared Return Performance: Home/exit requested at process t={:.3} ms",
                        crate::performance_process_elapsed_ms()
                    );
                    if !env.is_app_picker {
                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            ui_view::log_bounded_view_transition_history(env, "HomeButton");
                            ui_view::log_bounded_view_hierarchy(env, "HomeButton")
                        }));
                        // iOS kills apps that take more than a few seconds.
                        env.quit_deadline = Some(Instant::now() + Duration::from_secs(5));
                        // Some apps crash if they're quit while loading, as
                        // they would on a device. They're quitting anyway.
                        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            ui_application::notify_will_terminate(env)
                        }));
                        if let Err(e) = res {
                            // exit() unwinds the app to return to the picker.
                            if env.return_to_app_picker {
                                std::panic::resume_unwind(e);
                            }
                            log!("App crashed while quitting, returning to app picker anyway.");
                        }
                    }
                    env.return_to_app_picker = true;
                }
            }
            Event::TextInput(text_event) => {
                let responder = env.framework_state.uikit.ui_responder.first_responder;
                let class = msg![env; responder class];
                let ui_text_field_class = env.objc.get_known_class("UITextField", &mut env.mem);
                if !responder.is_null() && env.objc.class_is_subclass_of(class, ui_text_field_class)
                {
                    match text_event {
                        TextInputEvent::Text(text) => {
                            ui_view::ui_control::ui_text_field::handle_text(env, responder, text)
                        }
                        TextInputEvent::Backspace => {
                            ui_view::ui_control::ui_text_field::handle_backspace(env, responder)
                        }
                        TextInputEvent::Return => {
                            ui_view::ui_control::ui_text_field::handle_return(env, responder)
                        }
                    }
                }
            }
        }
    }

    let orientation_due = ui_device::handle_device_orientation(env);
    let accelerometer_due = ui_accelerometer::handle_accelerometer(env);
    match (orientation_due, accelerometer_due) {
        (Some(orientation), Some(accelerometer)) => Some(orientation.min(accelerometer)),
        (Some(due), None) | (None, Some(due)) => Some(due),
        (None, None) => None,
    }
}
