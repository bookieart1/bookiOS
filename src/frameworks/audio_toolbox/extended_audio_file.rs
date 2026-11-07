/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `ExtendedAudioFile.h` (Extended Audio File Services)
//!
//! Currently implemented as a dummy wrapper around Audio File Services.

// TODO: Audio format conversion

use super::audio_file::{
    kAudioFileBadPropertySizeError, kAudioFilePropertyDataFormat, kAudioFileReadPermission,
    property_size, AudioFileClose, AudioFileGetProperty, AudioFileID, AudioFileOpenURL,
    AudioFileReadBytes,
};
use super::audio_queue::is_supported_audio_format;
use super::audio_unit::AudioBufferList;
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::carbon_core::{eofErr, paramErr, OSStatus};
use crate::frameworks::core_audio_types::{
    debug_fourcc, fourcc, kAudioFormatFlagIsBigEndian, kAudioFormatFlagIsFloat,
    kAudioFormatFlagIsPacked, kAudioFormatFlagIsSignedInteger, kAudioFormatLinearPCM,
    AudioStreamBasicDescription,
};
use crate::frameworks::core_foundation::cf_url::CFURLRef;
use crate::frameworks::foundation::ns_url::to_rust_path;
use crate::mem::{guest_size_of, ConstPtr, ConstVoidPtr, MutPtr, MutVoidPtr, SafeRead};
use crate::Environment;
use std::collections::HashMap;

#[derive(Default)]
pub struct State {
    pub extended_audio_files: HashMap<ExtAudioFileRef, ExtAudioFileHostObject>,
}
impl State {
    pub fn get(framework_state: &mut crate::frameworks::State) -> &mut Self {
        &mut framework_state.audio_toolbox.extended_audio_file
    }
}

pub struct ExtAudioFileHostObject {
    guest_audio_file: AudioFileID,
    client_data_format: Option<AudioStreamBasicDescription>,
    current_bytes_read: i64,
    owns_audio_file: bool,
    conversion: ClientConversion,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum ClientConversion {
    #[default]
    Identity,
    Unsigned8ToSigned16,
}

#[repr(C, packed)]
pub struct OpaqueExtAudioFile {
    _filler: u8,
}
unsafe impl SafeRead for OpaqueExtAudioFile {}

type ExtAudioFileRef = MutPtr<OpaqueExtAudioFile>;

/// Usually a FourCC.
type ExtAudioFilePropertyID = u32;
const kExtAudioFileProperty_FileDataFormat: ExtAudioFilePropertyID = fourcc(b"ffmt");
const kExtAudioFileProperty_ClientDataFormat: ExtAudioFilePropertyID = fourcc(b"cfmt");
const kExtAudioFileProperty_FileLengthFrames: ExtAudioFilePropertyID = fourcc(b"#frm");
const kExtAudioFileProperty_AudioFile: ExtAudioFilePropertyID = fourcc(b"afil");

fn ExtAudioFileOpenURL(
    env: &mut Environment,
    in_url: CFURLRef,
    out_ext_audio_file: MutPtr<ExtAudioFileRef>,
) -> OSStatus {
    return_if_null!(in_url);

    let path = to_rust_path(env, in_url);
    log_dbg!(
        "ExtAudioFileOpenURL({:?} '{:?}', {:?})",
        in_url,
        path,
        out_ext_audio_file
    );

    let audio_file_ptr: MutPtr<AudioFileID> = env.mem.alloc(guest_size_of::<AudioFileID>()).cast();
    let res = AudioFileOpenURL(env, in_url, kAudioFileReadPermission, 0, audio_file_ptr);
    let guest_audio_file = env.mem.read(audio_file_ptr);
    env.mem.free(audio_file_ptr.cast());
    if res != 0 {
        log!(
            "ExtAudioFileOpenURL({:?} '{:?}', {:?}) failed, error: {:?}",
            in_url,
            path,
            out_ext_audio_file,
            res
        );
        return res;
    }

    let host_object = ExtAudioFileHostObject {
        guest_audio_file,
        client_data_format: None,
        current_bytes_read: 0,
        owns_audio_file: true,
        conversion: ClientConversion::Identity,
    };

    let guest_extended_audio_file = env.mem.alloc_and_write(OpaqueExtAudioFile { _filler: 0 });
    State::get(&mut env.framework_state)
        .extended_audio_files
        .insert(guest_extended_audio_file, host_object);

    env.mem.write(out_ext_audio_file, guest_extended_audio_file);

    0 // success
}

fn ExtAudioFileWrapAudioFileID(
    env: &mut Environment,
    in_file_id: AudioFileID,
    in_for_writing: bool,
    out_ext_audio_file: MutPtr<ExtAudioFileRef>,
) -> OSStatus {
    if in_file_id.is_null() || out_ext_audio_file.is_null() || in_for_writing {
        return paramErr;
    }
    if !env
        .framework_state
        .audio_toolbox
        .audio_file
        .audio_files
        .contains_key(&in_file_id)
    {
        return paramErr;
    }
    let host_object = ExtAudioFileHostObject {
        guest_audio_file: in_file_id,
        client_data_format: None,
        current_bytes_read: 0,
        owns_audio_file: false,
        conversion: ClientConversion::Identity,
    };
    let guest_extended_audio_file = env.mem.alloc_and_write(OpaqueExtAudioFile { _filler: 0 });
    State::get(&mut env.framework_state)
        .extended_audio_files
        .insert(guest_extended_audio_file, host_object);
    env.mem.write(out_ext_audio_file, guest_extended_audio_file);
    0
}

fn ExtAudioFileGetProperty(
    env: &mut Environment,
    in_ext_audio_file: ExtAudioFileRef,
    in_property_id: ExtAudioFilePropertyID,
    io_property_data_size: MutPtr<u32>,
    out_property_data: MutVoidPtr,
) -> OSStatus {
    return_if_null!(in_ext_audio_file);

    if in_property_id == kExtAudioFileProperty_FileLengthFrames {
        if env.mem.read(io_property_data_size) != guest_size_of::<i64>() {
            log!("Warning: ExtAudioFileGetProperty() failed");
            return kAudioFileBadPropertySizeError;
        }
        let guest_audio_file = State::get(&mut env.framework_state)
            .extended_audio_files
            .get(&in_ext_audio_file)
            .unwrap()
            .guest_audio_file;
        let audio_file = &env
            .framework_state
            .audio_toolbox
            .audio_file
            .audio_files
            .get(&guest_audio_file)
            .unwrap()
            .audio_file;
        let frames_per_packet =
            AudioStreamBasicDescription::from_audio_description(audio_file.audio_description())
                .frames_per_packet;
        let length_frames = (audio_file.packet_count() * u64::from(frames_per_packet)) as i64;
        log_dbg!(
            "ExtAudioFileGetProperty({:?}, kExtAudioFileProperty_FileLengthFrames) => {}",
            in_ext_audio_file,
            length_frames
        );
        env.mem.write(out_property_data.cast(), length_frames);
        return 0; // success
    }

    if in_property_id == kExtAudioFileProperty_AudioFile {
        if env.mem.read(io_property_data_size) != guest_size_of::<AudioFileID>() {
            log!("Warning: ExtAudioFileGetProperty() failed");
            return kAudioFileBadPropertySizeError;
        }
        let guest_audio_file = State::get(&mut env.framework_state)
            .extended_audio_files
            .get(&in_ext_audio_file)
            .unwrap()
            .guest_audio_file;
        env.mem.write(out_property_data.cast(), guest_audio_file);
        return 0; // success
    }

    let audio_file_property_id = match in_property_id {
        kExtAudioFileProperty_FileDataFormat => kAudioFilePropertyDataFormat,
        _ => unimplemented!(
            "Unimplemented property ID: {}",
            debug_fourcc(in_property_id)
        ),
    };

    let required_size = property_size(audio_file_property_id).unwrap();
    if env.mem.read(io_property_data_size) != required_size {
        log!("Warning: ExtAudioFileGetProperty() failed");
        return kAudioFileBadPropertySizeError;
    }

    let host_object = env
        .framework_state
        .audio_toolbox
        .extended_audio_file
        .extended_audio_files
        .get(&in_ext_audio_file)
        .unwrap();

    AudioFileGetProperty(
        env,
        host_object.guest_audio_file,
        audio_file_property_id,
        io_property_data_size,
        out_property_data,
    )
}

fn ExtAudioFileSetProperty(
    env: &mut Environment,
    in_ext_audio_file: ExtAudioFileRef,
    in_property_id: ExtAudioFilePropertyID,
    in_property_data_size: u32,
    in_property_data: ConstVoidPtr,
) -> OSStatus {
    return_if_null!(in_ext_audio_file);

    assert_eq!(in_property_id, kExtAudioFileProperty_ClientDataFormat);
    assert_eq!(
        in_property_data_size,
        guest_size_of::<AudioStreamBasicDescription>()
    );

    let audio_desc_ptr: ConstPtr<AudioStreamBasicDescription> = in_property_data.cast();
    let client_audio_desc = env.mem.read(audio_desc_ptr);
    log_dbg!("ExtAudioFileSetProperty {:?}", client_audio_desc);
    let format_id = client_audio_desc.format_id;
    assert_eq!(format_id, kAudioFormatLinearPCM);
    assert!(is_supported_audio_format(&client_audio_desc));

    let guest_audio_file = env
        .framework_state
        .audio_toolbox
        .extended_audio_file
        .extended_audio_files
        .get(&in_ext_audio_file)
        .unwrap()
        .guest_audio_file;
    let other_host_object = env
        .framework_state
        .audio_toolbox
        .audio_file
        .audio_files
        .get(&guest_audio_file)
        .unwrap();
    let audio_desc = AudioStreamBasicDescription::from_audio_description(
        other_host_object.audio_file.audio_description(),
    );
    let conversion = if audio_desc == client_audio_desc {
        ClientConversion::Identity
    } else if supports_unsigned8_to_signed16(audio_desc, client_audio_desc) {
        ClientConversion::Unsigned8ToSigned16
    } else {
        panic!(
            "Unsupported ExtAudioFile conversion {:?} -> {:?}",
            audio_desc, client_audio_desc
        );
    };
    let host_object = env
        .framework_state
        .audio_toolbox
        .extended_audio_file
        .extended_audio_files
        .get_mut(&in_ext_audio_file)
        .unwrap();
    assert!(host_object.client_data_format.is_none());
    host_object.client_data_format = Some(client_audio_desc);
    host_object.conversion = conversion;

    0 // success
}

fn ExtAudioFileRead(
    env: &mut Environment,
    in_ext_audio_file: ExtAudioFileRef,
    io_number_frames: MutPtr<u32>,
    io_data: MutPtr<AudioBufferList<1>>,
) -> OSStatus {
    return_if_null!(in_ext_audio_file);

    let mut audio_buffer_list = env.mem.read(io_data);
    let num_buffers = audio_buffer_list.number_buffers;
    assert_eq!(num_buffers, 1);

    let host_object = env
        .framework_state
        .audio_toolbox
        .extended_audio_file
        .extended_audio_files
        .get(&in_ext_audio_file)
        .unwrap();

    audio_buffer_list.buffers[0].number_channels =
        host_object.client_data_format.unwrap().channels_per_frame;

    let number_frames = env.mem.read(io_number_frames);
    let client_format = host_object.client_data_format.unwrap();
    let conversion = host_object.conversion;
    let guest_audio_file = host_object.guest_audio_file;
    let current_bytes_read = host_object.current_bytes_read;
    let source_format = AudioStreamBasicDescription::from_audio_description(
        env.framework_state
            .audio_toolbox
            .audio_file
            .audio_files
            .get(&guest_audio_file)
            .unwrap()
            .audio_file
            .audio_description(),
    );
    let source_bytes_requested = number_frames
        .checked_mul(source_format.bytes_per_frame)
        .unwrap();
    let destination_bytes_requested = number_frames
        .checked_mul(client_format.bytes_per_frame)
        .unwrap();
    if audio_buffer_list.buffers[0].data_byte_size < destination_bytes_requested {
        return paramErr;
    }
    let source_buffer = if conversion == ClientConversion::Identity {
        audio_buffer_list.buffers[0].data
    } else {
        env.mem.alloc(source_bytes_requested).cast()
    };
    let number_of_bytes_ptr = env.mem.alloc_and_write(source_bytes_requested);
    let res = AudioFileReadBytes(
        env,
        guest_audio_file,
        false,
        current_bytes_read,
        number_of_bytes_ptr,
        source_buffer,
    );
    let number_of_bytes_read = env.mem.read(number_of_bytes_ptr);
    env.mem.free(number_of_bytes_ptr.cast());
    if res != 0 {
        if conversion != ClientConversion::Identity {
            env.mem.free(source_buffer);
        }
        if res == eofErr {
            env.mem.write(io_number_frames, 0);
            return 0;
        }
        log!(
            "ExtAudioFileRead({:?}, {:?}, {:?}) failed, error: {:?}",
            in_ext_audio_file,
            io_number_frames,
            io_data,
            res
        );
        return res;
    }
    let frames_read = number_of_bytes_read
        .checked_div(source_format.bytes_per_frame)
        .unwrap();
    let destination_bytes_read = frames_read * client_format.bytes_per_frame;
    if conversion == ClientConversion::Unsigned8ToSigned16 {
        let converted = convert_unsigned8_to_signed16(
            env.mem
                .bytes_at(source_buffer.cast_const().cast(), number_of_bytes_read),
        );
        env.mem
            .bytes_at_mut(audio_buffer_list.buffers[0].data.cast(), destination_bytes_read)
            .copy_from_slice(&converted);
        env.mem.free(source_buffer);
    }
    env.mem.write(io_number_frames, frames_read);
    audio_buffer_list.buffers[0].data_byte_size = destination_bytes_read;

    let host_object = env
        .framework_state
        .audio_toolbox
        .extended_audio_file
        .extended_audio_files
        .get_mut(&in_ext_audio_file)
        .unwrap();
    host_object.current_bytes_read += number_of_bytes_read as i64;

    env.mem.write(io_data, audio_buffer_list);

    0 // success
}

fn supports_unsigned8_to_signed16(
    source: AudioStreamBasicDescription,
    destination: AudioStreamBasicDescription,
) -> bool {
    source.format_id == kAudioFormatLinearPCM
        && destination.format_id == kAudioFormatLinearPCM
        && (source.sample_rate - destination.sample_rate).abs() < 0.01
        && source.channels_per_frame == destination.channels_per_frame
        && source.bits_per_channel == 8
        && source.bytes_per_frame == source.channels_per_frame
        && source.format_flags & (kAudioFormatFlagIsFloat | kAudioFormatFlagIsBigEndian | kAudioFormatFlagIsSignedInteger) == 0
        && source.format_flags & kAudioFormatFlagIsPacked != 0
        && destination.bits_per_channel == 16
        && destination.bytes_per_frame == destination.channels_per_frame * 2
        && destination.format_flags & (kAudioFormatFlagIsFloat | kAudioFormatFlagIsBigEndian) == 0
        && destination.format_flags & (kAudioFormatFlagIsSignedInteger | kAudioFormatFlagIsPacked)
            == (kAudioFormatFlagIsSignedInteger | kAudioFormatFlagIsPacked)
}

fn convert_unsigned8_to_signed16(source: &[u8]) -> Vec<u8> {
    source
        .iter()
        .flat_map(|&sample| ((i16::from(sample) - 128) << 8).to_le_bytes())
        .collect()
}

fn ExtAudioFileSeek(
    env: &mut Environment,
    in_ext_audio_file: ExtAudioFileRef,
    in_frame_offset: i64,
) -> OSStatus {
    return_if_null!(in_ext_audio_file);

    let host_object = State::get(&mut env.framework_state)
        .extended_audio_files
        .get(&in_ext_audio_file)
        .unwrap();
    let guest_audio_file = host_object.guest_audio_file;
    // Reading is by byte, and the client format has to match the file's (see
    // ExtAudioFileSetProperty), so either gives the size of a frame.
    let bytes_per_frame = match host_object.client_data_format {
        Some(format) => format.bytes_per_frame,
        None => {
            let audio_file = &env
                .framework_state
                .audio_toolbox
                .audio_file
                .audio_files
                .get(&guest_audio_file)
                .unwrap()
                .audio_file;
            AudioStreamBasicDescription::from_audio_description(audio_file.audio_description())
                .bytes_per_frame
        }
    };
    assert!(
        bytes_per_frame != 0,
        "Can't seek in a compressed audio file"
    );

    log_dbg!(
        "ExtAudioFileSeek({:?}, {})",
        in_ext_audio_file,
        in_frame_offset
    );
    State::get(&mut env.framework_state)
        .extended_audio_files
        .get_mut(&in_ext_audio_file)
        .unwrap()
        .current_bytes_read = in_frame_offset * i64::from(bytes_per_frame);

    0 // success
}

fn ExtAudioFileDispose(env: &mut Environment, in_ext_audio_file: ExtAudioFileRef) -> OSStatus {
    return_if_null!(in_ext_audio_file);

    let host_object = env
        .framework_state
        .audio_toolbox
        .extended_audio_file
        .extended_audio_files
        .remove(&in_ext_audio_file)
        .unwrap();
    if host_object.owns_audio_file {
        let res = AudioFileClose(env, host_object.guest_audio_file);
        assert_eq!(res, 0); // success
    }
    env.mem.free(in_ext_audio_file.cast());

    0 // success
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(ExtAudioFileOpenURL(_, _)),
    export_c_func!(ExtAudioFileWrapAudioFileID(_, _, _)),
    export_c_func!(ExtAudioFileGetProperty(_, _, _, _)),
    export_c_func!(ExtAudioFileSetProperty(_, _, _, _)),
    export_c_func!(ExtAudioFileRead(_, _, _)),
    export_c_func!(ExtAudioFileSeek(_, _)),
    export_c_func!(ExtAudioFileDispose(_)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::ConstVoidPtr;

    #[test]
    fn dev_244_unsigned8_pcm_widens_to_signed16_endpoints() {
        assert_eq!(
            convert_unsigned8_to_signed16(&[0, 128, 255]),
            [0x00, 0x80, 0x00, 0x00, 0x00, 0x7f]
        );
    }

    #[test]
    fn dev_244_nonowning_extaudio_dispose_leaves_wrapped_audio_file_alive() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let audio_file: AudioFileID = env.mem.alloc(1).cast();
        let wrapper = env.mem.alloc_and_write(OpaqueExtAudioFile { _filler: 0 });
        State::get(&mut env.framework_state).extended_audio_files.insert(
            wrapper,
            ExtAudioFileHostObject {
                guest_audio_file: audio_file,
                client_data_format: None,
                current_bytes_read: 0,
                owns_audio_file: false,
                conversion: ClientConversion::Identity,
            },
        );

        assert_eq!(ExtAudioFileDispose(&mut env, wrapper), 0);
        assert!(env
            .mem
            .get_bytes_fallible(ConstVoidPtr::from_bits(audio_file.to_bits()), 1)
            .is_some());
        env.mem.free(audio_file.cast());
    }
}
