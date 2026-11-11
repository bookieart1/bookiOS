/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `MPMoviePlayerController` etc.

use crate::dyld::{ConstantExports, HostConstant};
use crate::frameworks::core_graphics::CGRect;
use crate::frameworks::foundation::{ns_string, ns_url, NSInteger, NSTimeInterval};
use crate::frameworks::uikit::ui_device::UIDeviceOrientation;
use crate::objc::{
    id, msg, msg_class, nil, objc_classes, release, retain, todo_objc_setter, ClassExports,
    HostObject, NSZonePtr,
};
use crate::Environment;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct State {
    active_player: Option<id>,
    /// Various apps (e.g. Crash Bandicoot Nitro Kart 3D and Spore Origins)
    /// create or start a player and await some kind of notification, but can't
    /// handle it if that notification happens immediately. This queue lets us
    /// delay such notifications until the app next returns to the run loop,
    /// which seems to be late enough.
    pending_notifications: VecDeque<(&'static str, id, Instant)>,
}
impl State {
    fn get(env: &mut Environment) -> &mut Self {
        &mut env.framework_state.media_player.movie_player
    }
}

type MPMovieScalingMode = NSInteger;
type MPMovieControlStyle = NSInteger;
type MPMovieSourceType = NSInteger;
const MPMovieSourceTypeUnknown: MPMovieSourceType = 0;

type MPMoviePlaybackState = NSInteger;
const MPMoviePlaybackStateStopped: MPMoviePlaybackState = 0;
type MPMovieRepeatMode = NSInteger;
const MPMovieRepeatModeNone: MPMovieRepeatMode = 0;

// Values might not be correct, but as these are linked symbol constants, it
// shouldn't matter.
pub const MPMoviePlayerPlaybackDidFinishNotification: &str =
    "MPMoviePlayerPlaybackDidFinishNotification";
/// Apparently an undocumented, private API. Spore Origins uses it.
pub const MPMoviePlayerContentPreloadDidFinishNotification: &str =
    "MPMoviePlayerContentPreloadDidFinishNotification";
pub const MPMoviePlayerScalingModeDidChangeNotification: &str =
    "MPMoviePlayerScalingModeDidChangeNotification";
pub const MPMoviePlayerLoadStateDidChangeNotification: &str =
    "MPMoviePlayerLoadStateDidChangeNotification";
pub const MPMoviePlayerDidExitFullscreenNotification: &str =
    "MPMoviePlayerDidExitFullscreenNotification";
// TODO: More notifications?
const MPMoviePlayerPlaybackDidFinishReasonUserInfoKey: &str =
    "MPMoviePlayerPlaybackDidFinishReasonUserInfoKey";

/// `NSNotificationName` values and other constants.
pub const CONSTANTS: ConstantExports = &[
    (
        "_MPMoviePlayerPlaybackDidFinishNotification",
        HostConstant::NSString(MPMoviePlayerPlaybackDidFinishNotification),
    ),
    (
        "_MPMoviePlayerContentPreloadDidFinishNotification",
        HostConstant::NSString(MPMoviePlayerContentPreloadDidFinishNotification),
    ),
    (
        "_MPMoviePlayerScalingModeDidChangeNotification",
        HostConstant::NSString(MPMoviePlayerScalingModeDidChangeNotification),
    ),
    (
        "_MPMoviePlayerLoadStateDidChangeNotification",
        HostConstant::NSString(MPMoviePlayerLoadStateDidChangeNotification),
    ),
    (
        "_MPMoviePlayerDidExitFullscreenNotification",
        HostConstant::NSString(MPMoviePlayerDidExitFullscreenNotification),
    ),
    (
        "_MPMoviePlayerPlaybackDidFinishReasonUserInfoKey",
        HostConstant::NSString(MPMoviePlayerPlaybackDidFinishReasonUserInfoKey),
    ),
];

struct MPMoviePlayerControllerHostObject {
    // NSURL *
    content_url: id,
    movie_source_type: MPMovieSourceType,
    should_autoplay: bool,
    repeat_mode: MPMovieRepeatMode,
    // UIView *, made when first asked for
    view: id,
    // UIView *, made when first asked for
    background_view: id,
}
impl HostObject for MPMoviePlayerControllerHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation MPMoviePlayerController: NSObject

// TODO: actual playback

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(MPMoviePlayerControllerHostObject {
        content_url: nil,
        movie_source_type: MPMovieSourceTypeUnknown,
        should_autoplay: true,
        repeat_mode: MPMovieRepeatModeNone,
        view: nil,
        background_view: nil,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithContentURL:(id)url { // NSURL*
    log!(
        "TODO: [(MPMoviePlayerController*){:?} initWithContentURL:{:?} ({:?})]",
        this,
        url,
        ns_url::to_rust_path(env, url),
    );

    retain(env, url);
    env.objc.borrow_mut::<MPMoviePlayerControllerHostObject>(this).content_url = url;

    // Act as if loading immediately completed (Spore Origins waits for this).
    State::get(env).pending_notifications.push_back(
        (MPMoviePlayerContentPreloadDidFinishNotification, this, Instant::now())
    );

    this
}

- (())dealloc {
    let host_obj = env.objc.borrow::<MPMoviePlayerControllerHostObject>(this);
    let (url, view, background_view) = (
        host_obj.content_url,
        host_obj.view,
        host_obj.background_view,
    );
    release(env, url);
    release(env, view);
    release(env, background_view);

    env.objc.dealloc_object(this, &mut env.mem);
}

- (id)contentURL {
    env.objc.borrow::<MPMoviePlayerControllerHostObject>(this).content_url
}

- (id)backgroundColor {
    msg_class![env; UIColor blackColor] // TODO
}
- (())setBackgroundColor:(id)color { // UIColor*
    todo_objc_setter!(this, color);
}

- (())setScalingMode:(MPMovieScalingMode)mode {
    todo_objc_setter!(this, mode);
}
- (())setUseApplicationAudioSession:(bool)use_session {
    todo_objc_setter!(this, use_session);
}
- (())setControlStyle:(MPMovieControlStyle)style {
    todo_objc_setter!(this, style);
}
- (())setFullscreen:(bool)fullsreen {
    todo_objc_setter!(this, fullsreen);
}
- (bool)shouldAutoplay {
    env.objc.borrow::<MPMoviePlayerControllerHostObject>(this).should_autoplay
}
- (())setShouldAutoplay:(bool)autoplay {
    env.objc.borrow_mut::<MPMoviePlayerControllerHostObject>(this).should_autoplay = autoplay;
}
- (())setInitialPlaybackTime:(NSTimeInterval)initial_time {
    todo_objc_setter!(this, initial_time);
}

- (MPMovieRepeatMode)repeatMode {
    env.objc.borrow::<MPMoviePlayerControllerHostObject>(this).repeat_mode
}
- (())setRepeatMode:(MPMovieRepeatMode)repeat_mode {
    env.objc.borrow_mut::<MPMoviePlayerControllerHostObject>(this).repeat_mode = repeat_mode;
}

// Whether the content is a file or a stream: only a hint for loading it.
- (MPMovieSourceType)movieSourceType {
    env.objc.borrow::<MPMoviePlayerControllerHostObject>(this).movie_source_type
}
- (())setMovieSourceType:(MPMovieSourceType)source_type {
    env.objc.borrow_mut::<MPMoviePlayerControllerHostObject>(this).movie_source_type =
        source_type;
}

- (id)view {
    let view = env.objc.borrow::<MPMoviePlayerControllerHostObject>(this).view;
    if view != nil {
        return view;
    }

    // MPMoviePlayerController's view property is non-nil on iOS. A number of
    // games configure or embed it before playback begins, so returning nil can
    // turn into a guest-side null dereference before any Objective-C error is
    // visible. The emulator still does not decode video, but it can provide the
    // real UIView contract and the expected screen-sized initial frame.
    let screen: id = msg_class![env; UIScreen mainScreen];
    let frame: CGRect = msg![env; screen applicationFrame];
    let view: id = msg_class![env; UIView alloc];
    let view: id = msg![env; view initWithFrame:frame];
    env.objc.borrow_mut::<MPMoviePlayerControllerHostObject>(this).view = view;
    log!(
        "[(MPMoviePlayerController*){:?} view] created {:?}",
        this,
        view
    );
    view
}

// For the game's own content behind the movie. TODO: it isn't shown, as
// there's no view to put it in yet.
- (id)backgroundView {
    let view = env.objc.borrow::<MPMoviePlayerControllerHostObject>(this).background_view;
    if view != nil {
        return view;
    }
    let view: id = msg_class![env; UIView alloc];
    let view: id = msg![env; view init];
    env.objc.borrow_mut::<MPMoviePlayerControllerHostObject>(this).background_view = view;
    view
}

- (MPMoviePlaybackState)playbackState {
    MPMoviePlaybackStateStopped // TODO
}

// Apparently an undocumented, private API, but Spore Origins uses it.
- (())setMovieControlMode:(NSInteger)_mode {
    // As this is undocumented and we don't have real video playback yet, let's
    // ignore it.
}

// Another undocumented one! But some apps may still use it :/
// https://stackoverflow.com/a/1390079/2241008
- (())setOrientation:(UIDeviceOrientation)_orientation animated:(bool)_animated {

}

// MPMediaPlayback implementation
- (())prepareToPlay {
    log!("TODO: [(MPMoviePlayerController*){:?} prepareToPlay]", this);
    // Some games never call play, and wait for the movie to finish.
    if env.objc.borrow::<MPMoviePlayerControllerHostObject>(this).should_autoplay {
        () = msg![env; this play];
    }
}

- (())play {
    log!("TODO: [(MPMoviePlayerController*){:?} play]", this);
    if let Some(old) = env.framework_state.media_player.movie_player.active_player {
        let _: () = msg![env; old stop];
    }
    assert!(env.framework_state.media_player.movie_player.active_player.is_none());
    // Movie player is retained by the runtime until it is stopped
    retain(env, this);
    env.framework_state.media_player.movie_player.active_player = Some(this);

    // Act as if playback immediately completed after 1 second
    // (various apps wait for this, such as BIA and Hero of Sparta).
    let notif = (MPMoviePlayerPlaybackDidFinishNotification, this, Instant::now().checked_add(Duration::from_millis(1000)).unwrap());
    for (name, obj, _) in &mut State::get(env).pending_notifications {
        // De-duplicate similar notifications. This can happen if app is calling
        // `play` twice on the same player object (case of NOVA2).
        if *name == MPMoviePlayerPlaybackDidFinishNotification && *obj == this {
            return;
        }
    }
    State::get(env).pending_notifications.push_back(notif);
}

- (())pause {
    log!("TODO: [(MPMoviePlayerController*){:?} pause]", this);
}

- (())stop {
    log!("TODO: [(MPMoviePlayerController*){:?} stop]", this);
    // Stopping an older controller must not tear down a newer active player.
    // Historical clients commonly keep controllers alive across transitions
    // and send a redundant late stop to the inactive one.
    State::get(env)
        .pending_notifications
        .retain(|&(_, object, _)| object != this);
    if State::get(env).active_player == Some(this) {
        State::get(env).active_player = None;
        release(env, this);
    }
}

@end

@implementation MPMoviePlayerViewController: UIViewController

- (id)initWithContentURL:(id)url {
    log!(
        "TODO: [(MPMoviePlayerViewController*){:?} initWithContentURL:{:?} ({:?})] -> nil",
        this,
        url,
        ns_url::to_rust_path(env, url),
    );
    release(env, this);
    nil // TODO
}

@end

};

/// For use by `NSRunLoop` via [super::handle_players]: check movie players'
/// status, send notifications if necessary.
pub(super) fn handle_players(env: &mut Environment) {
    let mut notifs_to_run = Vec::new();
    let pending_notifs = &mut State::get(env).pending_notifications;
    let mut i = 0;
    while i < pending_notifs.len() {
        let (name_str, object, time) = pending_notifs[i];
        if Instant::now() >= time {
            notifs_to_run.push((name_str, object));
            pending_notifs.swap_remove_back(i);
        } else {
            i += 1;
        }
    }
    for (name_str, object) in notifs_to_run {
        let name = ns_string::get_static_str(env, name_str);
        let center: id = msg_class![env; NSNotificationCenter defaultCenter];
        // TODO: should there be some user info attached?
        let _: () = msg![env; center postNotificationName:name object:object];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_244_movie_repeat_mode_is_paired_state() {
        let exported_names: Vec<_> = crate::frameworks::media_player::DYLIB
            .constant_exports
            .iter()
            .flat_map(|exports| exports.iter().map(|(name, _)| *name))
            .collect();
        assert!(exported_names.contains(
            &"_MPMusicPlayerControllerVolumeDidChangeNotification"
        ));

        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let player: id = msg_class![env; MPMoviePlayerController alloc];
        let initial: MPMovieRepeatMode = msg![env; player repeatMode];
        assert_eq!(initial, MPMovieRepeatModeNone);
        () = msg![env; player setRepeatMode:1i32];
        let repeated: MPMovieRepeatMode = msg![env; player repeatMode];
        assert_eq!(repeated, 1);
        release(env, player);
    }

    #[test]
    fn dev_245_stopping_inactive_movie_does_not_stop_newer_player() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let old: id = msg_class![env; MPMoviePlayerController alloc];
        let new: id = msg_class![env; MPMoviePlayerController alloc];
        () = msg![env; old play];
        () = msg![env; new play];
        () = msg![env; old stop];
        assert_eq!(State::get(env).active_player, Some(new));
        () = msg![env; new stop];
        release(env, old);
        release(env, new);
    }
}
