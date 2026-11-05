/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Audio file decoding and OpenAL bindings.
//!
//! The audio file decoding support is an abstraction over various libraries
//! (currently [hound] and [symphonia]), usage of which should be
//! confined to this module.
//!
//! Resources:
//! - [Apple Core Audio Format Specification 1.0](https://developer.apple.com/library/archive/documentation/MusicAudio/Reference/CAFSpec/CAF_intro/CAF_intro.html)

mod ima4;
pub mod openal;
mod symphonia_formats;

pub use symphonia_formats::{Mp3StreamDecoder, PacketStreamDecoder};

pub use ima4::decode_ima4;

use crate::fs::{Fs, GuestPath};
use std::io::Cursor;

#[derive(Debug)]
pub enum AudioFileOpenError {
    FileReadError,
    FileDecodeError,
}

#[derive(Debug)]
pub enum AudioFormat {
    LinearPcm {
        is_float: bool,
        is_little_endian: bool,
    },
}
/// A compressed format that [AudioFile] decodes to PCM when opening a file.
///
/// Apps still see the file as compressed, as on a real device: its packets
/// are variable-size, each holding [Self::frames_per_packet] frames. Unlike on
/// a real device though, the packets' contents are the decoded 16-bit PCM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompressedFormat {
    Aac,
    Mp3,
    Alac,
}

impl CompressedFormat {
    pub fn frames_per_packet(self) -> u32 {
        match self {
            CompressedFormat::Aac => 1024,
            CompressedFormat::Mp3 => 1152,
            CompressedFormat::Alac => 4096,
        }
    }
}

/// Fields have the same meanings as in the Core Audio Format's
/// Audio Description chunk, which is in turn similar to Core Audio Types'
/// `AudioStreamBasicDescription`.
#[derive(Debug)]
pub struct AudioDescription {
    /// Hz
    pub sample_rate: f64,
    pub format: AudioFormat,
    pub bytes_per_packet: u32,
    pub frames_per_packet: u32,
    pub channels_per_frame: u32,
    pub bits_per_channel: u32,
}

/// The inner file, and the file type as a FourCC (`AudioFileTypeID`), if known.
pub struct AudioFile(AudioFileInner, Option<[u8; 4]>);
enum AudioFileInner {
    Wave(hound::WavReader<Cursor<Vec<u8>>>),
    Symphonia(symphonia_formats::SymphoniaDecodedToPcm),
}

/// Guess a file's type (`AudioFileTypeID`) from its first bytes.
fn sniff_file_type(bytes: &[u8]) -> Option<[u8; 4]> {
    let at = |offset: usize, magic: &[u8]| bytes.get(offset..offset + magic.len()) == Some(magic);
    if at(0, b"RIFF") && at(8, b"WAVE") {
        Some(*b"WAVE")
    } else if at(0, b"caff") {
        Some(*b"caff")
    } else if at(0, b"FORM") && at(8, b"AIFC") {
        Some(*b"AIFC")
    } else if at(0, b"FORM") {
        Some(*b"AIFF")
    } else if at(4, b"ftyp") {
        Some(*b"m4af")
    } else if at(0, b"ID3") || (bytes.len() >= 2 && bytes[0] == 0xff && bytes[1] & 0xe0 == 0xe0) {
        // ADTS AAC also starts with 0xfff, but layer bits 00 mean it's not MP3.
        if bytes[1] & 0x06 == 0 && !at(0, b"ID3") {
            Some(*b"adts")
        } else {
            Some(*b"MPG3")
        }
    } else {
        None
    }
}

/// Compact format evidence for a file which none of the decoders accepted.
/// This deliberately reports only container metadata, never sample contents.
fn failed_audio_format_description(bytes: &[u8]) -> String {
    if bytes.get(..4) == Some(b"RIFF") && bytes.get(8..12) == Some(b"WAVE") {
        let mut pos = 12usize;
        while pos.checked_add(8).is_some_and(|end| end <= bytes.len()) {
            let size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
            let start = pos + 8;
            let Some(end) = start.checked_add(size) else {
                break;
            };
            if &bytes[pos..pos + 4] == b"fmt " && end <= bytes.len() && size >= 16 {
                let u16_at = |offset| {
                    u16::from_le_bytes(bytes[start + offset..start + offset + 2].try_into().unwrap())
                };
                let u32_at = |offset| {
                    u32::from_le_bytes(bytes[start + offset..start + offset + 4].try_into().unwrap())
                };
                return format!(
                    "RIFF/WAVE tag={:#06x} channels={} rate={} block-align={} bits={} bytes={}",
                    u16_at(0),
                    u16_at(2),
                    u32_at(4),
                    u16_at(12),
                    u16_at(14),
                    bytes.len()
                );
            }
            let Some(next) = end.checked_add(size & 1) else {
                break;
            };
            pos = next;
        }
        return format!("RIFF/WAVE without a valid fmt chunk; bytes={}", bytes.len());
    }

    if bytes.get(..4) == Some(b"FORM") && bytes.get(8..12) == Some(b"AIFC") {
        let mut pos = 12usize;
        while pos.checked_add(8).is_some_and(|end| end <= bytes.len()) {
            let size = u32::from_be_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
            let start = pos + 8;
            let Some(end) = start.checked_add(size) else {
                break;
            };
            if &bytes[pos..pos + 4] == b"COMM" && end <= bytes.len() && size >= 22 {
                let channels = u16::from_be_bytes(bytes[start..start + 2].try_into().unwrap());
                let frames = u32::from_be_bytes(bytes[start + 2..start + 6].try_into().unwrap());
                let bits = u16::from_be_bytes(bytes[start + 6..start + 8].try_into().unwrap());
                let compression = &bytes[start + 18..start + 22];
                return format!(
                    "AIFC compression={} channels={} frames={} bits={} bytes={}",
                    String::from_utf8_lossy(compression), channels, frames, bits, bytes.len()
                );
            }
            pos = end.saturating_add(size & 1);
        }
        return format!("AIFC without a valid COMM chunk; bytes={}", bytes.len());
    }

    let magic = bytes
        .iter()
        .take(12)
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("");
    format!("unrecognized/unsupported container magic={magic} bytes={}", bytes.len())
}

fn wav_to_pcm16<R: std::io::Read>(
    reader: hound::WavReader<R>,
) -> Result<symphonia_formats::SymphoniaDecodedToPcm, AudioFileOpenError> {
    let spec = reader.spec();
    let samples: Result<Vec<i16>, _> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .into_samples::<f32>()
            .map(|s| s.map(|s| (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16))
            .collect(),
        hound::SampleFormat::Int => {
            let shift = spec.bits_per_sample.saturating_sub(16);
            reader
                .into_samples::<i32>()
                .map(|s| s.map(|s| (s >> shift) as i16))
                .collect()
        }
    };
    let samples = samples.map_err(|_| AudioFileOpenError::FileDecodeError)?;
    Ok(symphonia_formats::SymphoniaDecodedToPcm {
        bytes: samples.iter().flat_map(|s| s.to_le_bytes()).collect(),
        sample_rate: spec.sample_rate,
        channels: spec.channels.into(),
        compressed_format: None,
    })
}

/// Decode ordinary integer PCM from a RIFF/WAVE file whose chunk layout is
/// legal but rejected by the stricter `hound` reader. Old iPhone games often
/// shipped assets made by permissive desktop tools, and Audio File Services
/// accepted chunks in an order that some modern readers do not.
///
/// This fallback deliberately supports only uncompressed 8- and 16-bit PCM.
/// It validates the format and frame size, ignores unknown bounded chunks and
/// never guesses a compressed codec.
fn riff_pcm_to_pcm16(bytes: &[u8]) -> Option<symphonia_formats::SymphoniaDecodedToPcm> {
    if bytes.get(..4)? != b"RIFF" || bytes.get(8..12)? != b"WAVE" {
        return None;
    }

    let mut format = None;
    let mut data = None;
    let mut pos = 12usize;
    while pos.checked_add(8)? <= bytes.len() {
        let chunk_type = bytes.get(pos..pos + 4)?;
        let declared_size =
            u32::from_le_bytes(bytes.get(pos + 4..pos + 8)?.try_into().ok()?) as usize;
        let start = pos + 8;
        let declared_end = start.checked_add(declared_size)?;

        if chunk_type == b"data" {
            // Some historical encoders left a stale data length in an
            // otherwise usable file. Audio File Services consumed the full
            // frames available at EOF, so do the same bounded clamping here.
            data = Some(bytes.get(start..declared_end.min(bytes.len()))?);
        } else {
            let chunk = bytes.get(start..declared_end)?;
            if chunk_type == b"fmt " && chunk.len() >= 16 {
                let u16_at = |offset| {
                    Some(u16::from_le_bytes(
                        chunk.get(offset..offset + 2)?.try_into().ok()?,
                    ))
                };
                let u32_at = |offset| {
                    Some(u32::from_le_bytes(
                        chunk.get(offset..offset + 4)?.try_into().ok()?,
                    ))
                };
                format = Some((
                    u16_at(0)?,
                    u16_at(2)?,
                    u32_at(4)?,
                    u16_at(12)?,
                    u16_at(14)?,
                ));
            }
        }

        if declared_end >= bytes.len() {
            break;
        }
        pos = declared_end.checked_add(declared_size & 1)?;
    }

    let (format_tag, channels, sample_rate, block_align, bits_per_sample) = format?;
    if format_tag != 1 || channels == 0 || sample_rate == 0 || !matches!(bits_per_sample, 8 | 16)
    {
        return None;
    }
    let expected_block_align = channels.checked_mul(bits_per_sample / 8)?;
    if block_align != expected_block_align || block_align == 0 {
        return None;
    }

    let data = data?;
    let complete_len = data.len() - data.len() % usize::from(block_align);
    let data = &data[..complete_len];
    let pcm16 = match bits_per_sample {
        8 => data
            .iter()
            .flat_map(|&sample| ((i16::from(sample) - 128) << 8).to_le_bytes())
            .collect(),
        16 => data.to_vec(),
        _ => unreachable!(),
    };
    Some(symphonia_formats::SymphoniaDecodedToPcm {
        bytes: pcm16,
        sample_rate,
        channels: channels.into(),
        compressed_format: None,
    })
}

/// Symphonia can't read 8-bit linear PCM in CAF files, so convert it to
/// 16-bit here. CAF's integer samples are always signed.
fn caf_pcm8_to_pcm16(bytes: &[u8]) -> Option<symphonia_formats::SymphoniaDecodedToPcm> {
    let u32_at = |at: usize| Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?));
    if bytes.get(..4)? != b"caff" {
        return None;
    }
    let mut desc = None;
    let mut pos = 8;
    while pos + 12 <= bytes.len() {
        let chunk_type = &bytes[pos..pos + 4];
        let size = i64::from_be_bytes(bytes[pos + 4..pos + 12].try_into().unwrap());
        let start = pos + 12;
        // A size of -1 means the chunk (always data) runs to the end.
        let end = if size < 0 {
            bytes.len()
        } else {
            start.checked_add(size.try_into().ok()?)?.min(bytes.len())
        };
        match chunk_type {
            b"desc" => {
                let rate = f64::from_bits(u64::from_be_bytes(
                    bytes.get(start..start + 8)?.try_into().ok()?,
                ));
                let format_id = bytes.get(start + 8..start + 12)?;
                let is_float = u32_at(start + 12)? & 1 != 0;
                let channels = u32_at(start + 24)?;
                let bits = u32_at(start + 28)?;
                if format_id != b"lpcm" || is_float || bits != 8 || channels == 0 {
                    return None;
                }
                desc = Some((rate, channels));
            }
            b"data" => {
                let (rate, channels) = desc?;
                // Skip the edit count.
                let samples = bytes.get(start + 4..end)?;
                return Some(symphonia_formats::SymphoniaDecodedToPcm {
                    bytes: samples
                        .iter()
                        .flat_map(|&s| (i16::from(s as i8) << 8).to_le_bytes())
                        .collect(),
                    sample_rate: rate as u32,
                    channels,
                    compressed_format: None,
                });
            }
            _ => (),
        }
        pos = end;
    }
    None
}

/// Reads linear PCM in AIFF files as 16-bit. Symphonia isn't used as it
/// rejects files iOS plays, such as ones whose SSND chunk is shorter than
/// COMM's frame count says (UConnect's sounds): the frames are read to the
/// end of the file here instead.
fn aiff_to_pcm16(bytes: &[u8]) -> Option<symphonia_formats::SymphoniaDecodedToPcm> {
    let u32_at = |at: usize| Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?));
    let form_type = bytes.get(8..12)?;
    if bytes.get(..4)? != b"FORM" || (form_type != b"AIFF" && form_type != b"AIFC") {
        return None;
    }
    let is_aifc = form_type == b"AIFC";
    let (mut comm, mut data_range) = (None, None);
    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let start = pos + 8;
        let size = usize::try_from(u32_at(pos + 4)?).ok()?;
        let end = start.checked_add(size)?;
        if end > bytes.len() {
            return None;
        }
        match &bytes[pos..pos + 4] {
            b"COMM" => {
                let channels = u16::from_be_bytes(bytes.get(start..start + 2)?.try_into().ok()?);
                let frames = u32_at(start + 2)?;
                let bits = u16::from_be_bytes(bytes.get(start + 6..start + 8)?.try_into().ok()?);
                // 80-bit extended float: 15-bit exponent, 64-bit mantissa.
                let exp = u16::from_be_bytes(bytes.get(start + 8..start + 10)?.try_into().ok()?);
                let mantissa =
                    u64::from_be_bytes(bytes.get(start + 10..start + 18)?.try_into().ok()?);
                let shift = (16383 + 63u32).checked_sub(u32::from(exp & 0x7fff))?;
                let rate = mantissa.checked_shr(shift)?;
                if channels == 0 || !(1..=32).contains(&bits) {
                    return None;
                }
                let compression = if is_aifc {
                    match bytes.get(start + 18..start + 22)? {
                        b"NONE" | b"twos" => *b"twos",
                        b"sowt" => *b"sowt",
                        b"ima4" => *b"ima4",
                        _ => return None,
                    }
                } else {
                    *b"twos"
                };
                comm = Some((channels, frames, bits, rate, compression));
            }
            b"SSND" => {
                let data_start = start
                    .checked_add(8)?
                    .checked_add(usize::try_from(u32_at(start)?).ok()?)?;
                if data_start > end {
                    return None;
                }
                data_range = Some(data_start..end);
            }
            _ => (),
        }
        // Chunks are padded to an even size.
        pos = end.checked_add(size & 1)?;
    }
    let (channels, frames, bits, rate, compression) = comm?;
    let samples = bytes.get(data_range?)?;
    if compression == *b"ima4" {
        if bits != 16 {
            return None;
        }
        let channels_usize = usize::from(channels);
        let packet_group_bytes = 34usize.checked_mul(channels_usize)?;
        let packet_groups = samples.len() / packet_group_bytes;
        let available_frames = packet_groups.checked_mul(64)?;
        let frame_count = usize::try_from(frames).ok()?.min(available_frames);
        let mut pcm = Vec::with_capacity(frame_count.checked_mul(channels_usize)?.checked_mul(2)?);
        let mut written_frames = 0usize;
        for packet_group in samples.chunks_exact(packet_group_bytes) {
            if written_frames >= frame_count {
                break;
            }
            let decoded: Vec<[i16; 64]> = packet_group
                .chunks_exact(34)
                .map(|packet| decode_ima4(packet.try_into().unwrap()))
                .collect();
            let frames_here = (frame_count - written_frames).min(64);
            for frame in 0..frames_here {
                for channel in &decoded {
                    pcm.extend_from_slice(&channel[frame].to_le_bytes());
                }
            }
            written_frames += frames_here;
        }
        return Some(symphonia_formats::SymphoniaDecodedToPcm {
            bytes: pcm,
            sample_rate: rate.try_into().ok()?,
            channels: channels.into(),
            compressed_format: None,
        });
    }
    let width = usize::from(bits).div_ceil(8);
    let len = usize::try_from(frames).ok()? * usize::from(channels) * width;
    let samples = &samples[..len.min(samples.len() / width * width)];
    let little_endian = compression == *b"sowt";
    Some(symphonia_formats::SymphoniaDecodedToPcm {
        // Samples are signed and big-endian: keep the top 16 bits.
        bytes: samples
            .chunks(width)
            .flat_map(|s| {
                if little_endian {
                    [s[0], s.get(1).copied().unwrap_or(0)]
                } else {
                    [s.get(1).copied().unwrap_or(0), s[0]]
                }
            })
            .collect(),
        sample_rate: rate.try_into().ok()?,
        channels: channels.into(),
        compressed_format: None,
    })
}

impl AudioFile {
    pub fn open_for_reading<P: AsRef<GuestPath>>(
        path: P,
        fs: &Fs,
    ) -> Result<Self, AudioFileOpenError> {
        // TODO: it would be better not to load the whole file at once
        let Ok(bytes) = fs.read(path.as_ref()) else {
            // TODO: Handle other FS related errors?
            return Err(AudioFileOpenError::FileReadError);
        };

        let format_description = failed_audio_format_description(&bytes);
        match Self::read_from_vec(bytes) {
            Ok(audio_file) => Ok(audio_file),
            Err(error) => {
                log!(
                    "Could not decode audio file at path {:?}: {:?}; {}.",
                    path.as_ref(),
                    error,
                    format_description
                );
                Err(AudioFileOpenError::FileReadError)
            }
        }
    }

    pub fn read_from_vec(bytes: Vec<u8>) -> Result<Self, AudioFileOpenError> {
        let file_type = sniff_file_type(&bytes);
        Self::read_from_vec_inner(bytes).map(|inner| AudioFile(inner, file_type))
    }

    fn read_from_vec_inner(bytes: Vec<u8>) -> Result<AudioFileInner, AudioFileOpenError> {
        // WavReader::new() consume the reader (in this case, a Cursor)
        // passed to it. This is a bit annoying considering we don't know
        // which is appropriate for the file. This is worked around here by
        // using temporary readers for checking if the file is the supported
        // format, then recreating the reader if that works.
        if let Ok(reader) = hound::WavReader::new(Cursor::new(&bytes)) {
            let spec = reader.spec();
            if spec.sample_format == hound::SampleFormat::Int
                && matches!(spec.bits_per_sample, 8 | 16)
            {
                let reader = hound::WavReader::new(Cursor::new(bytes)).unwrap();
                Ok(AudioFileInner::Wave(reader))
            } else {
                // Other sample formats are converted to 16-bit up front.
                Ok(AudioFileInner::Symphonia(wav_to_pcm16(reader)?))
            }
        // TODO: Real MP3/MP4/Non-linear PCM container handling. Currently we
        // are immediately decoding the entire file to PCM and acting as if
        // it's a PCM file, simply because because this is easier. Full MP3
        // support would require a lot of changes in Audio Toolbox.
        } else if let Some(pcm) = riff_pcm_to_pcm16(&bytes) {
            Ok(AudioFileInner::Symphonia(pcm))
        } else if let Some(pcm) = aiff_to_pcm16(&bytes) {
            Ok(AudioFileInner::Symphonia(pcm))
        } else if let Some(pcm) = caf_pcm8_to_pcm16(&bytes) {
            Ok(AudioFileInner::Symphonia(pcm))
        } else if let Ok(pcm) = symphonia_formats::decode_symphonia_to_pcm(Cursor::new(bytes)) {
            Ok(AudioFileInner::Symphonia(pcm))
        } else {
            Err(AudioFileOpenError::FileDecodeError)
        }
    }

    pub fn audio_description(&self) -> AudioDescription {
        match self.0 {
            AudioFileInner::Wave(ref wave_reader) => {
                let hound::WavSpec {
                    channels,
                    sample_rate,
                    bits_per_sample,
                    sample_format,
                } = wave_reader.spec();
                // Other formats were converted in read_from_vec().
                assert!(matches!(bits_per_sample, 8 | 16));
                assert!(sample_format == hound::SampleFormat::Int);

                AudioDescription {
                    sample_rate: sample_rate.into(),
                    format: AudioFormat::LinearPcm {
                        is_float: false,
                        is_little_endian: true,
                    },
                    bytes_per_packet: u32::from(channels * bits_per_sample / 8),
                    frames_per_packet: 1,
                    channels_per_frame: channels.into(),
                    bits_per_channel: bits_per_sample as u32,
                }
            }
            AudioFileInner::Symphonia(symphonia_formats::SymphoniaDecodedToPcm {
                sample_rate,
                channels,
                ..
            }) => AudioDescription {
                sample_rate: f64::from(sample_rate),
                format: AudioFormat::LinearPcm {
                    is_float: false,
                    is_little_endian: true,
                },
                bytes_per_packet: channels * 2,
                frames_per_packet: 1,
                channels_per_frame: channels,
                bits_per_channel: 16,
            },
        }
    }

    /// The file type as a FourCC (`AudioFileTypeID`), if known.
    pub fn file_type(&self) -> Option<[u8; 4]> {
        self.1
    }

    /// The compressed format the file was decoded from, if any.
    pub fn compressed_format(&self) -> Option<CompressedFormat> {
        match self.0 {
            AudioFileInner::Wave(_) => None,
            AudioFileInner::Symphonia(ref pcm) => pcm.compressed_format,
        }
    }

    /// For a file decoded from a compressed format: the size of a packet as
    /// apps see it (see [CompressedFormat]). The last packet may be shorter.
    pub fn compressed_packet_size(&self) -> Option<u32> {
        let format = self.compressed_format()?;
        let AudioDescription {
            bytes_per_packet, ..
        } = self.audio_description();
        Some(format.frames_per_packet() * bytes_per_packet)
    }

    /// For a file decoded from a compressed format: the number of packets as
    /// apps see them (see [CompressedFormat]).
    pub fn compressed_packet_count(&self) -> Option<u64> {
        let packet_size = u64::from(self.compressed_packet_size()?);
        Some(self.byte_count().div_ceil(packet_size))
    }

    /// For a file decoded from a compressed format: the byte offsets and sizes
    /// of up to `max_packets` packets from `start_packet`, stopping before the
    /// total size would exceed `max_bytes`, if given.
    pub fn compressed_packets(
        &self,
        start_packet: u64,
        max_packets: u32,
        max_bytes: Option<u32>,
    ) -> Vec<(u64, u32)> {
        let packet_size = self.compressed_packet_size().unwrap();
        let byte_count = self.byte_count();
        let mut packets = Vec::new();
        let mut total: u32 = 0;
        for i in 0..u64::from(max_packets) {
            let offset = (start_packet + i) * u64::from(packet_size);
            if offset >= byte_count {
                break;
            }
            let size = u64::from(packet_size).min(byte_count - offset) as u32;
            if max_bytes.is_some_and(|max_bytes| total + size > max_bytes) {
                break;
            }
            packets.push((offset, size));
            total += size;
        }
        packets
    }

    fn bytes_per_sample(&self) -> u64 {
        let AudioDescription {
            format,
            bytes_per_packet,
            frames_per_packet,
            channels_per_frame,
            ..
        } = self.audio_description();
        if !matches!(format, AudioFormat::LinearPcm { .. }) {
            panic!("{format:?} is a compressed format!");
        }
        ((bytes_per_packet / frames_per_packet) / channels_per_frame).into()
    }

    pub fn byte_count(&self) -> u64 {
        match self.0 {
            AudioFileInner::Wave(ref wave_reader) => {
                let sample_count = wave_reader.len(); // position-independent
                u64::from(sample_count) * self.bytes_per_sample()
            }
            AudioFileInner::Symphonia(symphonia_formats::SymphoniaDecodedToPcm {
                ref bytes,
                ..
            }) => bytes.len() as u64,
        }
    }

    pub fn packet_count(&self) -> u64 {
        match self.0 {
            AudioFileInner::Wave(_)
            | AudioFileInner::Symphonia(symphonia_formats::SymphoniaDecodedToPcm { .. }) => {
                // never variable-size
                self.byte_count() / u64::from(self.packet_size_fixed())
            }
        }
    }

    /// Returns the packet size if this audio format has a constant packet size,
    /// panics if not.
    pub fn packet_size_fixed(&self) -> u32 {
        let AudioDescription {
            bytes_per_packet, ..
        } = self.audio_description();
        assert!(bytes_per_packet != 0);
        bytes_per_packet
    }

    pub fn packet_size_upper_bound(&self) -> u32 {
        self.packet_size_fixed() // variable size not implemented
    }

    /// Read `buffer.len()` bytes of audio data from byte offset `offset`.
    /// Returns the number of bytes read.
    pub fn read_bytes(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize, ()> {
        match self.0 {
            AudioFileInner::Wave(_) => {
                let bytes_per_sample = self.bytes_per_sample();
                assert!(offset.is_multiple_of(bytes_per_sample));
                assert!(u64::try_from(buffer.len())
                    .unwrap()
                    .is_multiple_of(bytes_per_sample));

                let sample_count = u64::try_from(buffer.len()).unwrap() / bytes_per_sample;
                let sample_count: usize = sample_count.try_into().unwrap();

                let AudioFileInner::Wave(ref mut wave_reader) = self.0 else {
                    unreachable!()
                };

                let channels: u64 = wave_reader.spec().channels.into();
                // WavReader expects number of samples which are
                // independent of the number of channels here
                wave_reader
                    .seek((offset / (bytes_per_sample * channels)).try_into().unwrap())
                    .map_err(|_| ())?;

                let mut byte_offset = 0;
                for sample in wave_reader.samples().take(sample_count) {
                    let sample: i16 = sample.map_err(|_| ())?;
                    match bytes_per_sample {
                        // From the OpenAL docs: 8-bit PCM data is expressed as
                        // an unsigned value over the range 0 to 255, 128 being
                        // an audio output level of zero. Loaded wav samples
                        // must be converted to that from signed with 0 as
                        // output level 0.
                        1 => buffer[byte_offset] = (sample + 128) as u8,
                        2 => buffer[byte_offset..][..2].copy_from_slice(&sample.to_le_bytes()),
                        _ => todo!(),
                    }
                    byte_offset += bytes_per_sample as usize;
                }
                Ok(byte_offset)
            }
            AudioFileInner::Symphonia(symphonia_formats::SymphoniaDecodedToPcm {
                ref bytes,
                ..
            }) => {
                let bytes = bytes.get(offset as usize..).ok_or(())?;
                let bytes_to_read = buffer.len().min(bytes.len());
                let bytes = &bytes[..bytes_to_read];
                buffer[..bytes_to_read].copy_from_slice(bytes);
                Ok(bytes_to_read)
            }
        }
    }

    pub fn estimated_duration(&self) -> f64 {
        let AudioDescription {
            sample_rate,
            bytes_per_packet,
            frames_per_packet,
            ..
        } = self.audio_description();
        assert!(bytes_per_packet != 0); // TODO
        self.byte_count() as f64 * frames_per_packet as f64
            / (bytes_per_packet as f64 * sample_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_types() {
        assert_eq!(sniff_file_type(b"RIFF\0\0\0\0WAVEfmt "), Some(*b"WAVE"));
        assert_eq!(sniff_file_type(b"caff\0\x01\0\0"), Some(*b"caff"));
        assert_eq!(sniff_file_type(b"FORM\0\0\0\0AIFF"), Some(*b"AIFF"));
        assert_eq!(sniff_file_type(b"FORM\0\0\0\0AIFC"), Some(*b"AIFC"));
        assert_eq!(sniff_file_type(b"\0\0\0\x20ftypM4A "), Some(*b"m4af"));
        assert_eq!(sniff_file_type(b"ID3\x03\0"), Some(*b"MPG3"));
        assert_eq!(sniff_file_type(&[0xff, 0xfb, 0x90, 0x00]), Some(*b"MPG3"));
        assert_eq!(sniff_file_type(&[0xff, 0xf1, 0x50, 0x80]), Some(*b"adts"));
        assert_eq!(sniff_file_type(b"OggS"), None);
        assert_eq!(sniff_file_type(b""), None);
    }

    #[test]
    fn failed_wave_diagnostic_reports_bounded_format_metadata() {
        let bytes = [
            b'R', b'I', b'F', b'F', 0, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't',
            b' ', 16, 0, 0, 0, 2, 0, 1, 0, 0x22, 0x56, 0, 0, 0, 0, 0, 0, 0x00, 0x01, 4,
            0,
        ];
        assert_eq!(
            failed_audio_format_description(&bytes),
            "RIFF/WAVE tag=0x0002 channels=1 rate=22050 block-align=256 bits=4 bytes=36"
        );
    }

    #[test]
    fn compressed_packets() {
        // 2.5 AAC packets of 16-bit stereo
        let file = AudioFile(
            AudioFileInner::Symphonia(symphonia_formats::SymphoniaDecodedToPcm {
                bytes: vec![0; 2560 * 4],
                sample_rate: 44100,
                channels: 2,
                compressed_format: Some(CompressedFormat::Aac),
            }),
            None,
        );
        assert_eq!(file.compressed_packet_count(), Some(3));
        assert_eq!(
            file.compressed_packets(0, 10, None),
            vec![(0, 4096), (4096, 4096), (8192, 2048)]
        );
        assert_eq!(file.compressed_packets(0, 10, Some(8191)), vec![(0, 4096)]);
    }

    #[test]
    fn wav_24_bit_is_read_as_16_bit() {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 22050,
            bits_per_sample: 24,
            sample_format: hound::SampleFormat::Int,
        };
        let mut bytes = Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(&mut bytes, spec).unwrap();
        writer.write_sample(0x123456).unwrap();
        writer.finalize().unwrap();

        let mut file = AudioFile::read_from_vec(bytes.into_inner()).unwrap();
        let mut sample = [0; 2];
        assert_eq!(file.read_bytes(0, &mut sample), Ok(2));
        assert_eq!(i16::from_le_bytes(sample), 0x1234);
    }

    #[test]
    fn wav_pcm_fallback_accepts_data_before_fmt_and_converts_unsigned_8_bit() {
        let mut bytes = b"RIFF\0\0\0\0WAVEdata\x04\0\0\0\0\x80\xff\x40fmt \x10\0\0\0"
            .to_vec();
        bytes.extend_from_slice(&[
            1, 0, // PCM
            1, 0, // mono
            0x22, 0x56, 0, 0, // 22050 Hz
            0x22, 0x56, 0, 0, // 22050 bytes/s
            1, 0, // block align
            8, 0, // bits/sample
        ]);
        let riff_size = u32::try_from(bytes.len() - 8).unwrap().to_le_bytes();
        bytes[4..8].copy_from_slice(&riff_size);
        let pcm = riff_pcm_to_pcm16(&bytes).unwrap();
        assert_eq!(pcm.sample_rate, 22050);
        assert_eq!(pcm.channels, 1);
        assert_eq!(
            pcm.bytes,
            [0x00, 0x80, 0x00, 0x00, 0x00, 0x7f, 0x00, 0xc0]
        );
    }

    #[test]
    fn aifc_uncompressed_none_is_read_as_signed_big_endian_pcm() {
        let mut bytes = b"FORM\0\0\0\0AIFCCOMM\0\0\0\x16".to_vec();
        bytes.extend_from_slice(&[
            0, 1, // mono
            0, 0, 0, 2, // two frames
            0, 16, // 16-bit
            0x40, 0x0e, 0xac, 0x44, 0, 0, 0, 0, 0, 0, // 44100 extended
            b'N', b'O', b'N', b'E', // uncompressed AIFC
        ]);
        bytes.extend_from_slice(b"SSND\0\0\0\x0c\0\0\0\0\0\0\0\0\x12\x34\xfe\xdc");
        let form_size = u32::try_from(bytes.len() - 8).unwrap().to_be_bytes();
        bytes[4..8].copy_from_slice(&form_size);

        let pcm = aiff_to_pcm16(&bytes).unwrap();
        assert_eq!(pcm.sample_rate, 44100);
        assert_eq!(pcm.channels, 1);
        assert_eq!(pcm.bytes, [0x34, 0x12, 0xdc, 0xfe]);
    }

    #[test]
    fn aifc_ima4_packets_are_decoded_and_interleaved_as_pcm16() {
        let mut bytes = b"FORM\0\0\0\0AIFCCOMM\0\0\0\x16".to_vec();
        bytes.extend_from_slice(&[
            0, 1, // mono
            0, 0, 0, 64, // one 64-frame packet
            0, 16, // decoded sample size
            0x40, 0x0d, 0xac, 0x44, 0, 0, 0, 0, 0, 0, // 22050 extended
            b'i', b'm', b'a', b'4',
        ]);
        let packet = [0u8; 34];
        bytes.extend_from_slice(b"SSND\0\0\0\x2a\0\0\0\0\0\0\0\0");
        bytes.extend_from_slice(&packet);
        let form_size = u32::try_from(bytes.len() - 8).unwrap().to_be_bytes();
        bytes[4..8].copy_from_slice(&form_size);

        let pcm = aiff_to_pcm16(&bytes).unwrap();
        assert_eq!(pcm.sample_rate, 22050);
        assert_eq!(pcm.channels, 1);
        let expected: Vec<u8> = decode_ima4(&packet)
            .into_iter()
            .flat_map(i16::to_le_bytes)
            .collect();
        assert_eq!(pcm.bytes, expected);
    }
}
