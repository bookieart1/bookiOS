/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `AudioComponent.h` (Audio Component Services)

use std::collections::HashMap;
use std::time::Instant;

use crate::abi::GuestFunction;
use crate::audio::openal::al_types::ALuint;
use crate::dyld::FunctionExports;
use crate::environment::Environment;
use crate::export_c_func;
use crate::frameworks::carbon_core::{paramErr, OSStatus};
use crate::frameworks::core_audio_types::{
    debug_fourcc, fourcc, kAudioFormatFlagIsAlignedHigh, kAudioFormatFlagIsFloat,
    kAudioFormatFlagIsPacked, kAudioFormatFlagIsSignedInteger, kAudioFormatLinearPCM,
    AudioStreamBasicDescription,
};
use crate::mem::{ConstPtr, ConstVoidPtr, MutPtr, SafeRead};

const kAudioUnitType_Output: u32 = fourcc(b"auou");
const kAudioUnitSubType_RemoteIO: u32 = fourcc(b"rioc");
const kAudioUnitSubType_VoiceProcessingIO: u32 = fourcc(b"vpio");
const kAudioUnitType_Mixer: u32 = fourcc(b"aumx");
const kAudioUnitSubType_3DMixer: u32 = fourcc(b"3dem");
const kAudioUnitManufacturer_Apple: u32 = fourcc(b"appl");

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AudioComponentKind {
    Output,
    Mixer3D,
}

#[derive(Default)]
pub struct State {
    audio_components: HashMap<AudioComponent, AudioComponentKind>,
    pub audio_component_instances:
        HashMap<AudioComponentInstance, AudioComponentInstanceHostObject>,
}
impl State {
    pub fn get(framework_state: &mut crate::frameworks::State) -> &mut Self {
        &mut framework_state.audio_toolbox.audio_components
    }
}

#[derive(Clone)]
pub struct AudioComponentInstanceHostObject {
    pub kind: AudioComponentKind,
    pub started: bool,
    pub maximum_frames_per_slice: u32,
    pub global_stream_format: AudioStreamBasicDescription,
    pub input_stream_format: Option<AudioStreamBasicDescription>,
    pub output_stream_format: Option<AudioStreamBasicDescription>,
    pub render_callback: Option<AURenderCallbackStruct>,
    pub render_notifies: Vec<AURenderCallbackStruct>,
    pub last_render_time: Option<Instant>,
    /// For the render callback's timestamps.
    pub frames_rendered: u64,
    pub al_source: Option<ALuint>,
    pub is_running_handler: bool,
    pub connected_source: Option<AudioComponentInstance>,
    /// Number of input buses exposed by mixer units. Historical 3D Mixer
    /// clients resize this before installing per-bus callbacks.
    pub input_element_count: u32,
}
impl Default for AudioComponentInstanceHostObject {
    fn default() -> Self {
        // Default values obtained from an iPod Touch 4 running iOS 6.1.6
        // through a test app built targetting iOS 2.0
        AudioComponentInstanceHostObject {
            kind: AudioComponentKind::Output,
            started: false,
            // returning 1024 based on https://developer.apple.com/documentation/audiotoolbox/kaudiounitproperty_maximumframesperslice
            maximum_frames_per_slice: 1024,
            global_stream_format: AudioStreamBasicDescription {
                sample_rate: 44100.0,
                format_id: kAudioFormatLinearPCM,
                format_flags: kAudioFormatFlagIsFloat
                    | kAudioFormatFlagIsSignedInteger
                    | kAudioFormatFlagIsPacked
                    | kAudioFormatFlagIsAlignedHigh,
                bytes_per_packet: 4,
                frames_per_packet: 1,
                bytes_per_frame: 4,
                channels_per_frame: 2,
                bits_per_channel: 32,
                _reserved: 0,
            },
            input_stream_format: None,
            output_stream_format: None,
            render_callback: None,
            render_notifies: Vec::new(),
            last_render_time: None,
            frames_rendered: 0,
            al_source: None,
            is_running_handler: false,
            connected_source: None,
            input_element_count: 1,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
#[allow(dead_code)]
pub struct AURenderCallbackStruct {
    pub input_proc: AURenderCallback,
    pub input_proc_ref_con: ConstVoidPtr,
}
unsafe impl SafeRead for AURenderCallbackStruct {}

#[repr(C, packed)]
pub struct OpaqueAudioComponent {
    _pad: u8,
}
unsafe impl SafeRead for OpaqueAudioComponent {}

type AudioComponent = MutPtr<OpaqueAudioComponent>;

pub type AURenderCallback = GuestFunction;

#[repr(C, packed)]
pub struct OpaqueAudioComponentInstance {
    _pad: u8,
}
unsafe impl SafeRead for OpaqueAudioComponentInstance {}

pub type AudioComponentInstance = MutPtr<OpaqueAudioComponentInstance>;

#[repr(C, packed)]
struct AudioComponentDescription {
    component_type: u32,
    component_sub_type: u32,
    component_manufacturer: u32,
    component_flags: u32,
    component_flags_mask: u32,
}
unsafe impl SafeRead for AudioComponentDescription {}

fn AudioComponentFindNext(
    env: &mut Environment,
    in_component: AudioComponent,
    in_desc: ConstPtr<AudioComponentDescription>,
) -> AudioComponent {
    assert!(in_component.is_null());

    let audio_comp_descr = env.mem.read(in_desc);
    let component_type = audio_comp_descr.component_type;
    let component_sub_type = audio_comp_descr.component_sub_type;
    let component_manufacturer = audio_comp_descr.component_manufacturer;
    // VoiceProcessingIO is RemoteIO with echo cancellation for voice chat, so
    // it is treated as RemoteIO. Peggle and other OpenAL-era titles use the
    // historical Apple 3D mixer in front of RemoteIO. A field of 0 matches any
    // manufacturer.
    let kind = if component_type == kAudioUnitType_Output
        && (component_sub_type == kAudioUnitSubType_RemoteIO
            || component_sub_type == kAudioUnitSubType_VoiceProcessingIO)
        && matches!(component_manufacturer, 0 | kAudioUnitManufacturer_Apple)
    {
        Some(AudioComponentKind::Output)
    } else if component_type == kAudioUnitType_Mixer
        && component_sub_type == kAudioUnitSubType_3DMixer
        && matches!(component_manufacturer, 0 | kAudioUnitManufacturer_Apple)
    {
        Some(AudioComponentKind::Mixer3D)
    } else {
        None
    };
    let Some(kind) = kind else {
        // As on a device without such a component.
        log!(
            "Warning: AudioComponentFindNext() for unsupported component {} {} {}, returning NULL",
            debug_fourcc(component_type),
            debug_fourcc(component_sub_type),
            debug_fourcc(component_manufacturer)
        );
        return AudioComponent::null();
    };

    let state = State::get(&mut env.framework_state);
    if let Some((&component, _)) = state
        .audio_components
        .iter()
        .find(|(_, existing_kind)| **existing_kind == kind)
    {
        return component;
    }
    let out_component = env.mem.alloc_and_write(OpaqueAudioComponent { _pad: 0 });
    state.audio_components.insert(out_component, kind);

    log!(
        "TODO: AudioComponentFindNext({:?}, {:?}) -> {:?}",
        in_component,
        in_desc,
        out_component
    );
    out_component
}

fn AudioComponentInstanceNew(
    env: &mut Environment,
    in_component: AudioComponent,
    out_instance: MutPtr<AudioComponentInstance>,
) -> OSStatus {
    if out_instance.is_null() {
        return paramErr;
    }
    let Some(&kind) = State::get(&mut env.framework_state)
        .audio_components
        .get(&in_component)
    else {
        env.mem.write(out_instance, AudioComponentInstance::null());
        return paramErr;
    };
    let host_object = AudioComponentInstanceHostObject {
        kind,
        ..Default::default()
    };

    let guest_instance: AudioComponentInstance = env
        .mem
        .alloc_and_write(OpaqueAudioComponentInstance { _pad: 0 });
    State::get(&mut env.framework_state)
        .audio_component_instances
        .insert(guest_instance, host_object);

    env.mem.write(out_instance, guest_instance);

    let result = 0; // success
    log_dbg!(
        "AudioComponentInstanceNew({:?}, {:?}) -> {:?}",
        in_component,
        out_instance,
        result
    );
    result
}

fn AudioComponentInstanceDispose(
    env: &mut Environment,
    in_instance: AudioComponentInstance,
) -> OSStatus {
    let result = if in_instance.is_null()
        || State::get(&mut env.framework_state)
            .audio_component_instances
            .remove(&in_instance)
            .is_none()
    {
        paramErr
    } else {
        env.mem.free(in_instance.cast());
        0
    };
    log_dbg!(
        "AudioComponentInstanceDispose({:?}) -> {:?}",
        in_instance,
        result
    );
    result
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(AudioComponentFindNext(_, _)),
    export_c_func!(AudioComponentInstanceNew(_, _)),
    export_c_func!(AudioComponentInstanceDispose(_)),
];
