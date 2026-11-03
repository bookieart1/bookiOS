/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Bounded first implementation of the Audio Converter Services lifecycle.

use std::collections::HashMap;

use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::carbon_core::{paramErr, OSStatus};
use crate::frameworks::core_audio_types::{fourcc, AudioStreamBasicDescription};
use crate::mem::{ConstPtr, MutPtr, MutVoidPtr, Ptr};
use crate::Environment;

type AudioConverterRef = MutVoidPtr;

const K_AUDIO_CONVERTER_ERR_FORMAT_NOT_SUPPORTED: OSStatus = fourcc(b"fmt?") as i32;

#[derive(Default)]
pub struct State {
    converters: HashMap<AudioConverterRef, AudioConverterHostObject>,
}

#[derive(Copy, Clone)]
struct AudioConverterHostObject {
    source: AudioStreamBasicDescription,
    destination: AudioStreamBasicDescription,
}

fn AudioConverterNew(
    env: &mut Environment,
    source_format: ConstPtr<AudioStreamBasicDescription>,
    destination_format: ConstPtr<AudioStreamBasicDescription>,
    out_converter: MutPtr<AudioConverterRef>,
) -> OSStatus {
    if source_format.is_null() || destination_format.is_null() || out_converter.is_null() {
        return paramErr;
    }
    env.mem.write(out_converter, Ptr::null());
    let source = env.mem.read(source_format);
    let destination = env.mem.read(destination_format);
    log!(
        "AudioConverterNew source={:?} destination={:?}",
        source,
        destination
    );

    // Identity conversion has a complete and truthful lifecycle without
    // claiming codec or resampler support that the emulator does not have.
    if source != destination {
        log!("AudioConverterNew: conversion is not supported yet");
        return K_AUDIO_CONVERTER_ERR_FORMAT_NOT_SUPPORTED;
    }

    let converter = env.mem.alloc(1);
    let previous = env
        .framework_state
        .audio_toolbox
        .audio_converter
        .converters
        .insert(
            converter,
            AudioConverterHostObject {
                source,
                destination,
            },
        );
    assert!(previous.is_none());
    env.mem.write(out_converter, converter);
    0
}

fn AudioConverterReset(env: &mut Environment, converter: AudioConverterRef) -> OSStatus {
    let Some(host) = env
        .framework_state
        .audio_toolbox
        .audio_converter
        .converters
        .get(&converter)
    else {
        return paramErr;
    };
    let _formats = (host.source, host.destination);
    0
}

fn AudioConverterDispose(env: &mut Environment, converter: AudioConverterRef) -> OSStatus {
    if env
        .framework_state
        .audio_toolbox
        .audio_converter
        .converters
        .remove(&converter)
        .is_none()
    {
        return paramErr;
    }
    env.mem.free(converter);
    0
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(AudioConverterNew(_, _, _)),
    export_c_func!(AudioConverterReset(_)),
    export_c_func!(AudioConverterDispose(_)),
];
