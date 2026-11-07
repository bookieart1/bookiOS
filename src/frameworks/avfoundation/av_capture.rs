/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. */

use crate::dyld::{ConstantExports, HostConstant};

// These are NSString identity constants. Exporting them does not claim camera
// capture support; callers can compare/configure presets before deciding
// whether the unavailable capture-class surface is needed.
pub const CONSTANTS: ConstantExports = &[
    (
        "_AVCaptureSessionPresetLow",
        HostConstant::NSString("AVCaptureSessionPresetLow"),
    ),
    (
        "_AVCaptureSessionPresetMedium",
        HostConstant::NSString("AVCaptureSessionPresetMedium"),
    ),
    (
        "_AVCaptureSessionPreset640x480",
        HostConstant::NSString("AVCaptureSessionPreset640x480"),
    ),
    (
        "_AVCaptureSessionPreset1280x720",
        HostConstant::NSString("AVCaptureSessionPreset1280x720"),
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_248_capture_presets_export_typed_historical_names() {
        for (symbol, constant) in CONSTANTS {
            let HostConstant::NSString(value) = constant else {
                panic!("{symbol} was not an NSString")
            };
            assert_eq!(*value, symbol.trim_start_matches('_'));
        }
    }
}
