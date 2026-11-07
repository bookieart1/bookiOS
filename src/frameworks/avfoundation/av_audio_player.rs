/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! AVAudioPlayer
//!
//! Implemented using Audio Queue Services based on [the PlayingAudio example](https://developer.apple.com/library/archive/documentation/MusicAudio/Conceptual/AudioQueueProgrammingGuide/AQPlayback/PlayingAudio.html)

use crate::dyld::HostFunction;
use crate::frameworks::audio_toolbox::audio_file::{
    self, guest_audio_file_read_from_vec, kAudioFilePropertyDataFormat,
    kAudioFilePropertyPacketSizeUpperBound, kAudioFileReadPermission, AudioFileClose,
    AudioFileGetProperty, AudioFileID, AudioFileOpenURL, AudioFileReadPackets,
};
use crate::frameworks::audio_toolbox::audio_queue::{
    kAudioQueueParam_Volume, AudioQueueAllocateBuffer, AudioQueueBufferRef, AudioQueueDispose,
    AudioQueueEnqueueBuffer, AudioQueueGetParameter, AudioQueueNewOutput, AudioQueueOutputCallback,
    AudioQueuePause, AudioQueueRef, AudioQueueSetParameter, AudioQueueStart, AudioQueueStop,
};
use crate::frameworks::carbon_core::eofErr;
use crate::frameworks::core_audio_types::AudioStreamBasicDescription;
use crate::frameworks::core_foundation::cf_run_loop::kCFRunLoopCommonModes;
use crate::frameworks::foundation::ns_error::NSOSStatusErrorDomain;
use crate::frameworks::foundation::{ns_string, NSInteger, NSTimeInterval, NSUInteger};
use crate::mem::{guest_size_of, ConstVoidPtr, GuestUSize, MutPtr, MutVoidPtr, Ptr};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, retain, todo_objc_setter, Class,
    ClassExports, HostObject, NSZonePtr,
};
use crate::Environment;

const kNumberBuffers: usize = 3;

struct AVAudioPlayerHostObject {
    audio_file_url: id,
    output_callback: AudioQueueOutputCallback,
    audio_file_id: Option<AudioFileID>,
    audio_desc: Option<AudioStreamBasicDescription>,
    audio_queue: Option<AudioQueueRef>,
    audio_queue_buffers: Option<MutPtr<AudioQueueBufferRef>>,
    num_packets_to_read: u32,
    current_packet: i64,
    // The time set by calling setCurrentTime is stored here in case it's set
    // before prepareToPlay is called; so it can be applied when it's called
    set_current_time: NSTimeInterval,
    volume: f32,
    is_playing: bool,
    num_of_loops: NSInteger,
}
impl HostObject for AVAudioPlayerHostObject {}

fn log_pang_lifetime(env: &mut Environment, this: id, event: &str) {
    if env.bundle.bundle_identifier_optional() != Some("com.playerx.pang") {
        return;
    }
    let sequence = {
        let state = &mut env.framework_state.avfoundation;
        if state.pang_lifetime_diagnostic_events >= 96 {
            return;
        }
        state.pang_lifetime_diagnostic_events += 1;
        state.pang_lifetime_diagnostic_events
    };
    let (url, file, queue, buffers, playing) = {
        let object = env.objc.borrow::<AVAudioPlayerHostObject>(this);
        (
            object.audio_file_url,
            object.audio_file_id,
            object.audio_queue,
            object.audio_queue_buffers,
            object.is_playing,
        )
    };
    let refcount = env.objc.refcount_for_diagnostics(this);
    let regs = env.cpu.regs();
    log!(
        "Pang bounded AVAudioPlayer lifetime diagnostic: sequence={sequence} event={event} object={this:?} refcount={refcount:?} url={url:?} file={file:?} queue={queue:?} buffers={buffers:?} playing={playing} thread={} pc={:#010x} lr={:#010x}",
        env.current_thread,
        regs[crate::cpu::Cpu::PC],
        regs[crate::cpu::Cpu::LR]
    );
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation AVAudioPlayer: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let symb = "__touchHLE_AVAudioPlayerOutputBufferHelper";
    let hf: HostFunction = &(_touchHLE_AVAudioPlayerOutputBufferHelper as fn(&mut Environment, _, _, _) -> _);
    let callback = env
        .dyld
        .create_guest_function(&mut env.mem, symb, hf);

    let host_object = Box::new(AVAudioPlayerHostObject {
        audio_file_url: nil,
        output_callback: callback,
        audio_file_id: None,
        audio_desc: None,
        audio_queue: None,
        audio_queue_buffers: None,
        num_packets_to_read: 0,
        current_packet: 0,
        set_current_time: 0.0,
        volume: 1.0,
        is_playing: false,
        num_of_loops: 0
    });
    let object = env.objc.alloc_object(this, host_object, &mut env.mem);
    log_pang_lifetime(env, object, "alloc");
    object
}

- (id)initWithContentsOfURL:(id)url // NSURL*
                      error:(MutPtr<id>)out_error { // NSError**
    let path: id = msg![env; url path];
    let path_str = ns_string::to_rust_string(env, path);
    log_dbg!("[(AVAudioPlayer*){:?} initWithContentsOfURL:{:?} {} outError:{:?}]", this, url, path_str, out_error);

    retain(env, url);
    env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).audio_file_url = url;

    // Check for errors. Return nil and write them to error if there are
    let tmp_afi_ptr: MutPtr<AudioFileID> = env.mem.alloc(guest_size_of::<AudioFileID>()).cast();
    let status = AudioFileOpenURL(env, url, kAudioFileReadPermission, 0, tmp_afi_ptr) as NSInteger;
    let audio_file_id = env.mem.read(tmp_afi_ptr);
    env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).audio_file_id = Some(audio_file_id);
    env.mem.free(tmp_afi_ptr.cast());
    if status != 0 {
        if !out_error.is_null() {
            let domain = ns_string::get_static_str(env, NSOSStatusErrorDomain);
            let error = msg_class![env; NSError alloc];
            let error = msg![env; error initWithDomain:domain code:status userInfo:nil];
            autorelease(env, error);
            env.mem.write(out_error, error);
        }
        log_pang_lifetime(env, this, "init-url-error");
        release(env, this);
        return nil;
    }

    log_pang_lifetime(env, this, "init-url-success");
    this
}

- (id)initWithData:(id)data // NSData*
             error:(MutPtr<id>)out_error { // NSError**
    let bytes: ConstVoidPtr = msg![env; data bytes];
    let length: NSUInteger = msg![env; data length];
    let data_vec = env
        .mem
        .bytes_at(bytes.cast(), length)
        .to_vec();

    assert_eq!(env.objc.borrow::<AVAudioPlayerHostObject>(this).audio_file_url, nil);

    match guest_audio_file_read_from_vec(env, data_vec) {
        Ok(audio_file_id) => {
            assert!(env.objc.borrow::<AVAudioPlayerHostObject>(this).audio_file_id.is_none());
            env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).audio_file_id = Some(audio_file_id);
            log_pang_lifetime(env, this, "init-data-success");
            this
        }
        Err(_) => {
            let domain = ns_string::get_static_str(env, NSOSStatusErrorDomain);
            let error = msg_class![env; NSError alloc];
            let code = -1; // TODO: set a proper code
            let error = msg![env; error initWithDomain:domain code:code userInfo:nil];
            autorelease(env, error);
            env.mem.write(out_error, error);

            log_pang_lifetime(env, this, "init-data-error");
            release(env, this);
            nil
        }
    }
}

- (())setDelegate:(id)delegate {
    todo_objc_setter!(this, delegate);
}
- (id)url {
    // Retained by URL initialization and preserved by stop. Data-backed
    // players have no URL; reading the property does not transfer ownership.
    env.objc.borrow::<AVAudioPlayerHostObject>(this).audio_file_url
}
- (())setMeteringEnabled:(bool)enabled {
    todo_objc_setter!(this, enabled);
}

- (f32)volume {
    let aq_ref = env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).audio_queue;
    if aq_ref.is_none() {
        // TODO: is it correct? can we always return it instead of querying?
        return env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).volume;
    }

    let tmp: MutPtr<f32> = env.mem.alloc(guest_size_of::<f32>()).cast();
    let status = AudioQueueGetParameter(env, aq_ref.unwrap(), kAudioQueueParam_Volume, tmp);
    assert_eq!(status, 0);
    let res = env.mem.read(tmp);
    env.mem.free(tmp.cast());
    res
}
- (())setVolume:(f32)volume {
    let host_object = env.objc.borrow_mut::<AVAudioPlayerHostObject>(this);
    host_object.volume = volume;
    if let Some(aq_ref) = host_object.audio_queue {
        let status = AudioQueueSetParameter(env, aq_ref, kAudioQueueParam_Volume, volume);
        assert_eq!(status, 0);
    }
}

- (())prepareToPlay {
    log_pang_lifetime(env, this, "prepare-enter");
    let audio_queue = env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).audio_queue;
    if audio_queue.is_some() {
        return;
    }

    let audio_file_id = env.objc.borrow::<AVAudioPlayerHostObject>(this).audio_file_id.unwrap();
    let callback = env.objc.borrow::<AVAudioPlayerHostObject>(this).output_callback;

    let size = guest_size_of::<AudioStreamBasicDescription>();
    let tmp_size_ptr: MutPtr<GuestUSize> = env.mem.alloc(guest_size_of::<GuestUSize>()).cast();
    env.mem.write(tmp_size_ptr, size);
    let tmp_data_ptr: MutPtr<AudioStreamBasicDescription> = env.mem.alloc(size).cast();
    let status = AudioFileGetProperty(
        env, audio_file_id, kAudioFilePropertyDataFormat, tmp_size_ptr, tmp_data_ptr.cast()
    );
    assert_eq!(status, 0);
    assert_eq!(size, env.mem.read(tmp_size_ptr));
    let audio_desc = env.mem.read(tmp_data_ptr);
    log_dbg!("audio_desc {:?}", audio_desc);
    env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).audio_desc = Some(audio_desc);

    let aq_ref_ptr: MutPtr<AudioQueueRef> = env.mem.alloc(guest_size_of::<AudioQueueRef>()).cast();
    let common_modes = ns_string::get_static_str(env, kCFRunLoopCommonModes);
    let status = AudioQueueNewOutput(
        env, tmp_data_ptr.cast_const(), callback, this.cast(),
        Ptr::null(), common_modes, 0, aq_ref_ptr
    );
    assert_eq!(status, 0);
    let aq_ref = env.mem.read(aq_ref_ptr);
    env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).audio_queue = Some(aq_ref);

    // Reapply the previously set current time and volume in case
    // setVolume/setCurrentTime were called before prepareToPlay
    let volume = env.objc.borrow::<AVAudioPlayerHostObject>(this).volume;
    () = msg![env; this setVolume:volume];
    let set_current_time = env.objc.borrow::<AVAudioPlayerHostObject>(this).set_current_time;
    () = msg![env; this setCurrentTime:set_current_time];

    let size = guest_size_of::<u32>();
    env.mem.write(tmp_size_ptr, size);
    let prop_size_ptr: MutPtr<u32> = env.mem.alloc(size).cast();
    let status = AudioFileGetProperty(
        env, audio_file_id, kAudioFilePropertyPacketSizeUpperBound, tmp_size_ptr, prop_size_ptr.cast()
    );
    assert_eq!(status, 0);
    assert_eq!(size, env.mem.read(tmp_size_ptr));
    let prop_size = env.mem.read(prop_size_ptr);

    let (buffer_byte_size, num_packets_to_read) = derive_buffer_size(audio_desc, prop_size, 0.5);
    env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).num_packets_to_read = num_packets_to_read;

    let buffers: MutPtr<AudioQueueBufferRef> = env.mem.alloc(kNumberBuffers as GuestUSize * guest_size_of::<AudioQueueBufferRef>()).cast();
    env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).audio_queue_buffers = Some(buffers);

    env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).is_playing = true;
    for i in 0..kNumberBuffers {
        let status = AudioQueueAllocateBuffer(env, aq_ref, buffer_byte_size, buffers + i as u32);
        assert_eq!(status, 0);

        _touchHLE_AVAudioPlayerOutputBufferHelper(env, this.cast(), aq_ref, env.mem.read(buffers + i as u32));
    }
    env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).is_playing = false;

    env.mem.free(tmp_size_ptr.cast());
    env.mem.free(aq_ref_ptr.cast());
    env.mem.free(tmp_data_ptr.cast());
    log_pang_lifetime(env, this, "prepare-complete");
}

- (bool)isPlaying {
    log_pang_lifetime(env, this, "is-playing");
    env.objc.borrow::<AVAudioPlayerHostObject>(this).is_playing
}

- (bool)play {
    () = msg![env; this prepareToPlay];

    let aq_ref = env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).audio_queue.unwrap();

    env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).is_playing = true;

    let status = AudioQueueStart(env, aq_ref, Ptr::null());
    assert_eq!(status, 0);

    log_pang_lifetime(env, this, "play");
    true
}

- (())pause {
    env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).is_playing = false;
    if let Some(aq_ref) = env.objc.borrow::<AVAudioPlayerHostObject>(this).audio_queue {
        AudioQueuePause(env, aq_ref);
    }
    log_pang_lifetime(env, this, "pause");
}

- (())stop {
    log_pang_lifetime(env, this, "stop-enter");
    let &mut AVAudioPlayerHostObject {
        audio_queue,
        audio_queue_buffers,
        ..
    } = env.objc.borrow_mut::<AVAudioPlayerHostObject>(this);
    if audio_queue.is_none() {
        // already being stopped
        log_pang_lifetime(env, this, "stop-no-queue");
        return;
    }
    AudioQueueDispose(env, audio_queue.unwrap(), true);
    env.mem.free(audio_queue_buffers.unwrap().cast());

    let &AVAudioPlayerHostObject { audio_file_url, output_callback, num_of_loops, audio_file_id, .. } = env.objc.borrow(this);
    *env.objc.borrow_mut::<AVAudioPlayerHostObject>(this) = AVAudioPlayerHostObject {
        audio_file_url,
        output_callback,
        num_of_loops,
        audio_file_id,
        audio_desc: None,
        audio_queue: None,
        audio_queue_buffers: None,
        num_packets_to_read: 0,
        current_packet: 0,
        set_current_time: 0.0,
        volume: 1.0,
        is_playing: false
    };
    log_pang_lifetime(env, this, "stop-complete");
}

- (())setNumberOfLoops:(NSInteger)numberOfLoops {
    log_dbg!("[(AVAudioPlayer *) {:?} setNumberOfLoops:{:?}]", this, numberOfLoops);
    env.objc.borrow_mut::<AVAudioPlayerHostObject>(this).num_of_loops = numberOfLoops;
}

- (())dealloc {
    log_pang_lifetime(env, this, "dealloc-enter");
    () = msg![env; this stop];
    let &AVAudioPlayerHostObject {audio_file_url, audio_file_id, ..} = env.objc.borrow(this);
    release(env, audio_file_url);
    if let Some(audio_file_id) = audio_file_id {
        AudioFileClose(env, audio_file_id);
    }
    log_pang_lifetime(env, this, "dealloc-before-object-release");
    env.objc.dealloc_object(this, &mut env.mem)
}

- (NSTimeInterval)currentTime {
    let host_object = env.objc.borrow::<AVAudioPlayerHostObject>(this);
    let current_time = if let Some(audio_desc) = host_object.audio_desc {
        let current_frame = (host_object.current_packet as f64) * (audio_desc.frames_per_packet as f64);
        current_frame / audio_desc.sample_rate
    } else {
        0.0
    };
    log_dbg!("[(AVAudioPlayer *) {:?} currentTime] -> {:?}", this, current_time);
    current_time
}
- (())setCurrentTime:(NSTimeInterval)currentTime {
    // TODO: Support setting current time before having an audio description
    let host_object = env.objc.borrow_mut::<AVAudioPlayerHostObject>(this);
    host_object.set_current_time = currentTime;
    if let (Some(audio_desc), Some(audio_file_id)) = (host_object.audio_desc, host_object.audio_file_id) {
        let total_packets = audio_file::State::get(&mut env.framework_state).audio_files.get(&audio_file_id).unwrap().audio_file.packet_count();
        let total_frames = total_packets * audio_desc.frames_per_packet as u64;
        let new_current_frame = audio_desc.sample_rate * currentTime;
        if new_current_frame < 0.0 || new_current_frame > total_frames as f64 {
            host_object.current_packet = 0;
        } else {
            host_object.current_packet = (new_current_frame / (audio_desc.frames_per_packet as f64)) as i64;
        }
    }
    log_dbg!("[(AVAudioPlayer *) {:?} setCurrentTime: {}]", this, currentTime);
}

- (NSTimeInterval)duration {
    let host_object = env.objc.borrow::<AVAudioPlayerHostObject>(this);
    let audio_file_id = host_object.audio_file_id.unwrap();
    let audio_file = &audio_file::State::get(&mut env.framework_state)
        .audio_files
        .get(&audio_file_id)
        .unwrap()
        .audio_file;
    audio_file.estimated_duration()
}

@end

};

// Listing 3-7 from `Deriving a playback audio queue buffer size`
// from the Apple's guide
fn derive_buffer_size(
    audio_desc: AudioStreamBasicDescription,
    max_packet_size: u32,
    seconds: f64,
) -> (u32, u32) {
    let mut out_buffer_size;

    const max_buffer_size: u32 = 0x50000;
    const min_buffer_size: u32 = 0x4000;

    if audio_desc.frames_per_packet != 0 {
        let num_packets_to_time =
            audio_desc.sample_rate / audio_desc.frames_per_packet as f64 * seconds;
        out_buffer_size = num_packets_to_time as u32 * max_packet_size;
    } else {
        out_buffer_size = if max_buffer_size > max_packet_size {
            max_buffer_size
        } else {
            max_packet_size
        }
    }

    if out_buffer_size > max_buffer_size && out_buffer_size > max_packet_size {
        out_buffer_size = max_buffer_size
    } else if out_buffer_size < min_buffer_size {
        out_buffer_size = min_buffer_size
    }

    let out_num_packets_to_read = out_buffer_size / max_packet_size;
    (out_buffer_size, out_num_packets_to_read)
}

/// (*void)(void *in_user_data, AudioQueueRef in_aq, AudioQueueBufferRef in_buf)
fn _touchHLE_AVAudioPlayerOutputBufferHelper(
    env: &mut Environment,
    in_user_data: MutVoidPtr,
    in_aq: AudioQueueRef,
    in_buf: AudioQueueBufferRef,
) {
    let av_audio_player: id = in_user_data.cast();
    let class: Class = msg![env; av_audio_player class];
    log_dbg!(
        "_touchHLE_AVAudioPlayerOutputBufferHelper on object of class: {}",
        env.objc.get_class_name(class)
    );
    let audio_player_class = env.objc.get_known_class("AVAudioPlayer", &mut env.mem);
    assert!(env.objc.class_is_subclass_of(class, audio_player_class));

    let &AVAudioPlayerHostObject {
        audio_file_id,
        audio_queue,
        num_packets_to_read,
        current_packet,
        is_playing,
        ..
    } = env.objc.borrow(av_audio_player);
    let aq = audio_queue.unwrap();
    assert_eq!(aq, in_aq);

    if !is_playing {
        return;
    }

    let num_bytes_ptr: MutPtr<u32> = env.mem.alloc(guest_size_of::<u32>()).cast();
    let num_packets_ptr: MutPtr<u32> = env.mem.alloc(guest_size_of::<u32>()).cast();
    env.mem.write(num_packets_ptr, num_packets_to_read);
    let mut audio_queue_buffer = env.mem.read(in_buf);
    let status = AudioFileReadPackets(
        env,
        audio_file_id.unwrap(),
        false,
        num_bytes_ptr,
        Ptr::null(),
        current_packet,
        num_packets_ptr,
        audio_queue_buffer.audio_data,
    );
    let num_packets = env.mem.read(num_packets_ptr);
    let num_bytes = env.mem.read(num_bytes_ptr);
    env.mem.free(num_packets_ptr.cast());
    env.mem.free(num_bytes_ptr.cast());

    if num_packets > 0 {
        assert!(status == 0 || status == eofErr);
        audio_queue_buffer.audio_data_byte_size = num_bytes;
        env.mem.write(in_buf, audio_queue_buffer);
        let status = AudioQueueEnqueueBuffer(env, aq, in_buf, 0, Ptr::null());
        assert_eq!(status, 0);
        env.objc
            .borrow_mut::<AVAudioPlayerHostObject>(av_audio_player)
            .current_packet = current_packet + num_packets as i64;
    } else {
        // Reading compressed packets at the end gives no error.
        assert!(status == 0 || status == eofErr);
        let number_of_loops = env
            .objc
            .borrow::<AVAudioPlayerHostObject>(av_audio_player)
            .num_of_loops;
        if number_of_loops == 0 {
            let status = AudioQueueStop(env, aq, false);
            assert_eq!(status, 0);
            env.objc
                .borrow_mut::<AVAudioPlayerHostObject>(av_audio_player)
                .is_playing = false;
        } else {
            if number_of_loops > 0 {
                env.objc
                    .borrow_mut::<AVAudioPlayerHostObject>(av_audio_player)
                    .num_of_loops -= 1;
            }
            env.objc
                .borrow_mut::<AVAudioPlayerHostObject>(av_audio_player)
                .current_packet = 0;
            _touchHLE_AVAudioPlayerOutputBufferHelper(env, in_user_data, in_aq, in_buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_backed_audio_player_has_no_url() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        // NSData's initializer forwards five ABI arguments, so this fixture
        // needs guest stack space in addition to its synthetic thread.
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let mut wav = std::io::Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(
            &mut wav,
            hound::WavSpec {
                channels: 1,
                sample_rate: 22050,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .unwrap();
        writer.write_sample(0i16).unwrap();
        writer.finalize().unwrap();
        let wav = wav.into_inner();
        let length = wav.len() as u32;
        let bytes = env.mem.alloc(length);
        env.mem
            .bytes_at_mut(bytes.cast(), length)
            .copy_from_slice(&wav);
        let data: id = msg_class![env; NSData dataWithBytesNoCopy:bytes length:length];
        let player: id = msg_class![env; AVAudioPlayer alloc];
        let error: MutPtr<id> = env.mem.alloc(4).cast();
        env.mem.write(error, nil);
        let player: id = msg![env; player initWithData:data error:error];
        assert_ne!(player, nil);
        let url: id = msg![env; player url];
        assert_eq!(url, nil);
        () = msg![env; player stop];
        let url: id = msg![env; player url];
        assert_eq!(url, nil);
        release(env, player);
        release(env, pool);
        env.mem.free(error.cast());
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn url_getter_borrows_stored_url_and_survives_stop() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let path = ns_string::get_static_str(env, "/fixture.wav");
        let url: id = msg_class![env; NSURL fileURLWithPath:path];
        let player: id = msg_class![env; AVAudioPlayer alloc];
        let absent: id = msg![env; player url];
        assert_eq!(absent, nil);
        // Seed the same retained state as URL initialization, without opening
        // a host audio device or claiming to exercise filesystem decoding.
        retain(env, url);
        env.objc
            .borrow_mut::<AVAudioPlayerHostObject>(player)
            .audio_file_url = url;
        let count = env.objc.get_refcount(url);
        for _ in 0..3 {
            let actual: id = msg![env; player url];
            assert_eq!(actual, url);
            assert_eq!(env.objc.get_refcount(url), count);
            () = msg![env; player stop];
        }
        release(env, player);
        assert_eq!(env.objc.get_refcount(url).get(), count.get() - 1);
        release(env, pool);
    }
}
