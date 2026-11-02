/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIFont`.

use super::ui_graphics::UIGraphicsGetCurrentContext;
use crate::font::{Font, TextAlignment, WrapMode};
use crate::frameworks::core_graphics::cg_bitmap_context::CGBitmapContextDrawer;
use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::ns_array;
use crate::frameworks::foundation::ns_string::{from_rust_string, to_rust_string};
use crate::frameworks::foundation::NSInteger;
use crate::objc::{
    autorelease, id, msg, msg_super, objc_classes, retain, ClassExports, HostObject,
};
use crate::Environment;
use std::collections::HashMap;
use std::ops::Range;

#[derive(Default)]
pub(super) struct State {
    fonts: HashMap<FontKind, Font>,
    /// UIKit-created UIFont flyweights. The cache owns one retain on each
    /// immutable object while callers still receive normal autoreleased
    /// references. Several older apps keep system-font pointers across an
    /// autorelease-pool drain, matching the long-lived instances used by the
    /// historical UIKit implementation.
    ui_fonts: HashMap<(id, FontKind, u32, String), id>,
    sans_regular_ja: Option<Font>,
    sans_bold_ja: Option<Font>,
    marker_felt: Option<Font>,
    /// The names of the fonts the app bundles, in `fonts` as
    /// [FontKind::App], or [None] if they're not loaded yet.
    app_font_names: Option<Vec<Vec<String>>>,
    /// Recently deallocated fonts, retained only as bounded diagnostic data.
    deallocated_fonts: Vec<DeallocatedFont>,
}
impl State {
    fn get_font_by_kind(&mut self, font_kind: FontKind) -> &Font {
        self.fonts
            .entry(font_kind)
            .or_insert_with(|| new_system_font(font_kind))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum FontKind {
    MonoRegular,
    MonoBold,
    MonoBoldItalic,
    MonoItalic,
    SansRegular,
    SansBold,
    SansBoldItalic,
    SansItalic,
    SerifRegular,
    SerifBold,
    SerifBoldItalic,
    SerifItalic,
    MarkerFelt,
    /// A font the app bundles, by its index in `UIAppFonts`.
    App(usize),
}

#[derive(Copy, Clone)]
struct DeallocatedFont {
    object: id,
    size: CGFloat,
    kind: FontKind,
}

struct UIFontHostObject {
    size: CGFloat,
    kind: FontKind,
    name: String,
}
impl HostObject for UIFontHostObject {}

/// Line break mode.
///
/// This is put here for convenience since it's font-related.
/// Apple puts it in its own header, also in UIKit.
pub type UILineBreakMode = NSInteger;
pub const UILineBreakModeWordWrap: UILineBreakMode = 0;
pub const UILineBreakModeCharacterWrap: UILineBreakMode = 1;
#[allow(dead_code)]
pub const UILineBreakModeClip: UILineBreakMode = 2;
#[allow(dead_code)]
pub const UILineBreakModeHeadTruncation: UILineBreakMode = 3;
pub const UILineBreakModeTailTruncation: UILineBreakMode = 4;
#[allow(dead_code)]
pub const UILineBreakModeMiddleTruncation: UILineBreakMode = 5;

/// Text alignment.
///
/// This is put here for convenience since it's font-related.
/// Apple puts it in its own header, also in UIKit.
pub type UITextAlignment = NSInteger;
pub const UITextAlignmentLeft: UITextAlignment = 0;
pub const UITextAlignmentCenter: UITextAlignment = 1;
pub const UITextAlignmentRight: UITextAlignment = 2;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIFont: NSObject

// Values are checked against iPhone 3GS, iOS 4.0.1
+ (CGFloat)labelFontSize {
    17.0
}
+ (CGFloat)buttonFontSize {
    18.0
}
+ (CGFloat)smallSystemFontSize {
    12.0
}
+ (CGFloat)systemFontSize {
    14.0
}

+ (id)familyNames {
    font_string_array(env, &[
        "Courier",
        "AppleGothic",
        "Arial",
        "STHeiti TC",
        "Hiragino Kaku Gothic ProN",
        "Courier New",
        "Zapfino",
        "Arial Unicode MS",
        "STHeiti SC",
        "American Typewriter",
        "Helvetica",
        "Marker Felt",
        "Helvetica Neue",
        "DB LCD Temp",
        "Verdana",
        "Times New Roman",
        "Georgia",
    ])
}

+ (id)fontNamesForFamilyName:(id)familyName {
    let family_name = to_rust_string(env, familyName);
    let names: &[&str] = match family_name.as_ref() {
        "Courier" => &["Courier", "Courier-BoldOblique", "Courier-Oblique", "Courier-Bold"],
        "AppleGothic" => &["AppleGothic"],
        "Arial" => &["ArialMT", "Arial-BoldMT", "Arial-BoldItalicMT", "Arial-ItalicMT"],
        "STHeiti TC" => &["STHeitiTC-Light", "STHeitiTC-Medium"],
        "Hiragino Kaku Gothic ProN" => &["HiraKakuProN-W6", "HiraKakuProN-W3"],
        "Courier New" => &["CourierNewPS-BoldMT", "CourierNewPS-ItalicMT", "CourierNewPS-BoldItalicMT", "CourierNewPSMT"],
        "Zapfino" => &["Zapfino"],
        "Arial Unicode MS" => &["ArialUnicodeMS"],
        "STHeiti SC" => &["STHeitiSC-Medium", "STHeitiSC-Light"],
        "American Typewriter" => &["AmericanTypewriter", "AmericanTypewriter-Bold"],
        "Helvetica" => &["Helvetica-Oblique", "Helvetica-BoldOblique", "Helvetica", "Helvetica-Bold"],
        "Marker Felt" => &["MarkerFelt-Thin", "MarkerFelt-Wide"],
        "Helvetica Neue" => &["HelveticaNeue", "HelveticaNeue-Bold"],
        "DB LCD Temp" => &["DBLCDTempBlack"],
        "Verdana" => &["Verdana-Bold", "Verdana-BoldItalic", "Verdana", "Verdana-Italic"],
        "Times New Roman" => &["TimesNewRomanPSMT", "TimesNewRomanPS-BoldMT", "TimesNewRomanPS-BoldItalicMT", "TimesNewRomanPS-ItalicMT"],
        "Georgia" => &["Georgia-Bold", "Georgia", "Georgia-BoldItalic", "Georgia-Italic"],
        _ => &[],
    };
    font_string_array(env, names)
}

+ (id)systemFontOfSize:(CGFloat)size {
    cached_font(env, this, FontKind::SansRegular, size, "Helvetica".to_owned())
}
+ (id)boldSystemFontOfSize:(CGFloat)size {
    cached_font(env, this, FontKind::SansBold, size, "Helvetica-Bold".to_owned())
}
+ (id)italicSystemFontOfSize:(CGFloat)size {
    cached_font(env, this, FontKind::SansItalic, size, "Helvetica-Oblique".to_owned())
}
+ (id)fontWithName:(id)fontName // NSString*
            size:(CGFloat)fontSize {
    let font_name = to_rust_string(env, fontName).to_string();
    let (kind, resolved_name) = get_app_font(env, &font_name)
        .map(|kind| (kind, font_name.clone()))
        .or_else(|| {
            get_equivalent_font(&font_name)
                .map(|kind| (kind, font_name.clone()))
        })
        .unwrap_or_else(|| {
            log!("No replacement found for font {}. Using system font instead.", font_name);
            (FontKind::SansRegular, "Helvetica".to_owned())
        });
    cached_font(env, this, kind, fontSize, resolved_name)
}

- (id)fontWithSize:(CGFloat)size {
    let kind = env.objc.borrow::<UIFontHostObject>(this).kind;
    let name = env.objc.borrow::<UIFontHostObject>(this).name.clone();
    let class = msg![env; this class];
    cached_font(env, class, kind, size, name)
}

- (id)fontName {
    let name = env.objc.borrow::<UIFontHostObject>(this).name.clone();
    let name = from_rust_string(env, name);
    autorelease(env, name)
}

- (CGFloat)pointSize {
    let host_object = env.objc.borrow::<UIFontHostObject>(this);
    host_object.size
}

- (CGFloat)ascender {
    let host_object = env.objc.borrow::<UIFontHostObject>(this);
    let font = env.framework_state.uikit.ui_font.get_font_by_kind(host_object.kind);
    font.ascent(host_object.size)
}
- (CGFloat)descender {
    let host_object = env.objc.borrow::<UIFontHostObject>(this);
    let font = env.framework_state.uikit.ui_font.get_font_by_kind(host_object.kind);
    font.descent(host_object.size)
}
- (CGFloat)leading {
    let host_object = env.objc.borrow::<UIFontHostObject>(this);
    let font = env.framework_state.uikit.ui_font.get_font_by_kind(host_object.kind);
    font.ascent(host_object.size).ceil() + (-font.descent(host_object.size)).ceil()
}

- (CGFloat)lineHeight {
    // This is calculated based on the documentation:
    // https://developer.apple.com/library/archive/documentation/TextFonts/Conceptual/CocoaTextArchitecture/FontHandling/FontHandling.html
    let ascender: CGFloat = msg![env; this ascender];
    let descender: CGFloat = msg![env; this descender];
    let leading: CGFloat = msg![env; this leading];
    assert!(descender <= 0.0);
    ascender + leading - descender
}

- (())dealloc {
    let font = env.objc.borrow::<UIFontHostObject>(this);
    let diagnostic = DeallocatedFont {
        object: this,
        size: font.size,
        kind: font.kind,
    };
    let state = &mut env.framework_state.uikit.ui_font;
    // A guest can still over-release a cached object. Do not leave a stale
    // pointer in the flyweight table if that happens.
    state.ui_fonts.retain(|_, cached| *cached != this);
    let diagnostics = &mut state.deallocated_fonts;
    const MAX_DEALLOCATED_FONT_DIAGNOSTICS: usize = 32;
    if diagnostics.len() == MAX_DEALLOCATED_FONT_DIAGNOSTICS {
        diagnostics.remove(0);
    }
    diagnostics.push(diagnostic);
    msg_super![env; this dealloc]
}

@end

};

/// Build the immutable arrays returned by UIFont's class discovery methods.
/// The list matches the font families exposed by the historical iOS 2 font
/// table already used by `get_equivalent_font` below.
fn font_string_array(env: &mut Environment, strings: &[&str]) -> id {
    let strings = strings
        .iter()
        .map(|string| from_rust_string(env, (*string).to_owned()))
        .collect();
    let array = ns_array::from_vec(env, strings);
    autorelease(env, array)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg_class;

    #[test]
    fn font_name_and_resized_font_preserve_public_identity() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let font: id = msg_class![env; UIFont boldSystemFontOfSize:17.0f32];
        let name: id = msg![env; font fontName];
        assert_eq!(to_rust_string(env, name), "Helvetica-Bold");
        let resized: id = msg![env; font fontWithSize:24.0f32];
        let resized_name: id = msg![env; resized fontName];
        assert_eq!(to_rust_string(env, resized_name), "Helvetica-Bold");
    }

    #[test]
    fn family_names_and_family_members_match_supported_font_table() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let families: id = msg_class![env; UIFont familyNames];
        let family_count: u32 = msg![env; families count];
        assert!(family_count >= 17);

        let family = from_rust_string(env, "Helvetica".to_owned());
        let names: id = msg_class![env; UIFont fontNamesForFamilyName:family];
        let names_count: u32 = msg![env; names count];
        assert_eq!(names_count, 4);
        let regular: id = msg![env; names objectAtIndex:2u32];
        let bold: id = msg![env; names objectAtIndex:3u32];
        assert_eq!(to_rust_string(env, regular), "Helvetica");
        assert_eq!(to_rust_string(env, bold), "Helvetica-Bold");

        let missing = from_rust_string(env, "Not Installed".to_owned());
        let missing_names: id = msg_class![env; UIFont fontNamesForFamilyName:missing];
        let missing_count: u32 = msg![env; missing_names count];
        assert_eq!(missing_count, 0);
    }
}

/// Return an immutable UIFont flyweight at +0 while the framework cache owns
/// one retain. This models the long-lived system-font objects used by old
/// UIKit without fabricating a font after a genuine use-after-free.
fn cached_font(
    env: &mut Environment,
    class: id,
    kind: FontKind,
    size: CGFloat,
    name: String,
) -> id {
    let key = (class, kind, size.to_bits(), name.clone());
    let cached = env
        .framework_state
        .uikit
        .ui_font
        .ui_fonts
        .get(&key)
        .copied();
    if let Some(font) = cached {
        if env.objc.get_host_object(font).is_some() {
            let returned = retain(env, font);
            return autorelease(env, returned);
        }
        env.framework_state.uikit.ui_font.ui_fonts.remove(&key);
    }

    let host_object = UIFontHostObject { size, kind, name };
    let font = env
        .objc
        .alloc_object(class, Box::new(host_object), &mut env.mem);
    env.framework_state.uikit.ui_font.ui_fonts.insert(key, font);
    let returned = retain(env, font);
    autorelease(env, returned)
}

/// Find a font the app bundles by name, loading them if needed.
fn get_app_font(env: &mut Environment, name: &str) -> Option<FontKind> {
    if env.framework_state.uikit.ui_font.app_font_names.is_none() {
        let mut all_names = Vec::new();
        for (i, file) in env.bundle.app_fonts().into_iter().enumerate() {
            let path = env.bundle.bundle_path().join(file);
            let loaded = env.fs.read(&path).ok().and_then(Font::from_app_file);
            let Some((font, names)) = loaded else {
                log!("Warning: couldn't load app font {:?}", file);
                all_names.push(Vec::new());
                continue;
            };
            log_dbg!("Loaded app font {:?}, names {:?}", file, names);
            let state = &mut env.framework_state.uikit.ui_font;
            state.fonts.insert(FontKind::App(i), font);
            all_names.push(names);
        }
        env.framework_state.uikit.ui_font.app_font_names = Some(all_names);
    }
    let all_names = env.framework_state.uikit.ui_font.app_font_names.as_ref();
    all_names
        .unwrap()
        .iter()
        .position(|names| names.iter().any(|n| n == name))
        .map(FontKind::App)
}

/// Create an independent Core Graphics font for a public font name. Core
/// Graphics fonts own their font data, unlike UIFont's cached rendering
/// objects, so this deliberately constructs a fresh Font.
pub(crate) fn new_font_for_name(env: &mut Environment, name: &str) -> Option<Font> {
    for file in env.bundle.app_fonts() {
        let path = env.bundle.bundle_path().join(file);
        let Some((font, names)) = env.fs.read(&path).ok().and_then(Font::from_app_file) else {
            continue;
        };
        if names.iter().any(|candidate| candidate == name) {
            return Some(font);
        }
    }

    let kind = match name {
        // Historical iPhone system fonts for which touchHLE already uses its
        // bundled metric-compatible families. UIFont falls back to SansRegular
        // for these today; CGFont must still return a real font for a valid
        // installed name.
        "Courier" | "CourierNewPSMT" => FontKind::MonoRegular,
        "Courier-Bold" | "CourierNewPS-BoldMT" => FontKind::MonoBold,
        "Courier-Oblique" | "CourierNewPS-ItalicMT" => FontKind::MonoItalic,
        "Courier-BoldOblique" | "CourierNewPS-BoldItalicMT" => FontKind::MonoBoldItalic,
        "Helvetica" | "HelveticaNeue" | "Verdana" | "AppleGothic" | "STHeitiTC-Light"
        | "STHeitiTC-Medium" | "STHeitiSC-Light" | "STHeitiSC-Medium" | "HiraKakuProN-W3"
        | "ArialUnicodeMS" => FontKind::SansRegular,
        "Helvetica-Bold" | "HelveticaNeue-Bold" | "Verdana-Bold" | "HiraKakuProN-W6" => {
            FontKind::SansBold
        }
        "Helvetica-Oblique" | "Verdana-Italic" => FontKind::SansItalic,
        "Helvetica-BoldOblique" | "Verdana-BoldItalic" => FontKind::SansBoldItalic,
        "Georgia" => FontKind::SerifRegular,
        "Georgia-Bold" => FontKind::SerifBold,
        "Georgia-Italic" | "Zapfino" => FontKind::SerifItalic,
        "Georgia-BoldItalic" => FontKind::SerifBoldItalic,
        "DBLCDTempBlack" => FontKind::MonoRegular,
        _ => get_equivalent_font(name)?,
    };
    Some(new_system_font(kind))
}

fn new_system_font(kind: FontKind) -> Font {
    match kind {
        FontKind::MonoRegular => Font::mono_regular(),
        FontKind::MonoBold => Font::mono_bold(),
        FontKind::MonoBoldItalic => Font::mono_bold_italic(),
        FontKind::MonoItalic => Font::mono_italic(),
        FontKind::SansRegular => Font::sans_regular(),
        FontKind::SansBold => Font::sans_bold(),
        FontKind::SansBoldItalic => Font::sans_bold_italic(),
        FontKind::SansItalic => Font::sans_italic(),
        FontKind::SerifRegular => Font::serif_regular(),
        FontKind::SerifBold => Font::serif_bold(),
        FontKind::SerifBoldItalic => Font::serif_bold_italic(),
        FontKind::SerifItalic => Font::serif_italic(),
        FontKind::MarkerFelt => Font::marker_felt(),
        FontKind::App(_) => unreachable!("app fonts are loaded from their bundle data"),
    }
}

fn convert_line_break_mode(ui_mode: UILineBreakMode) -> WrapMode {
    match ui_mode {
        UILineBreakModeWordWrap => WrapMode::Word,
        UILineBreakModeCharacterWrap => WrapMode::Char,
        // TODO: support this properly; fake support is so that UILabel works,
        // which has this as its default line break mode
        UILineBreakModeTailTruncation => WrapMode::Word,
        _ => unimplemented!("TODO: line break mode {}", ui_mode),
    }
}

#[rustfmt::skip]
fn get_font<'a>(state: &'a mut State, kind: FontKind, text: &str) -> &'a Font {
    // The default fonts (see font.rs) are the Liberation family, which are a
    // good substitute for Helvetica, the iPhone OS system font. Unfortunately,
    // there is no CJK support in these fonts. To support Super Monkey Ball in
    // Japanese, let's fall back to Noto Sans JP when necessary.
    // FIXME: This heuristic is incomplete and a proper font fallback system
    // should be used instead.
    for c in text.chars() {
        let c = c as u32;
        if (0x3000..=0x30FF).contains(&c) || // JA punctuation, kana
           (0xFF00..=0xFFEF).contains(&c) || // full-width/half-width chars
           (0x4e00..=0x9FA0).contains(&c) || // various kanji
           (0x3400..=0x4DBF).contains(&c) { // more kanji
            match kind {
                // CJK has no italic equivalent
                FontKind::MonoRegular | FontKind::MonoItalic | FontKind::SansRegular | FontKind::SansItalic | FontKind::SerifRegular | FontKind::SerifItalic | FontKind::App(_) => {
                    if state.sans_regular_ja.is_none() {
                        state.sans_regular_ja = Some(Font::sans_regular_ja());
                    }
                    return state.sans_regular_ja.as_ref().unwrap();
                },
                FontKind::MonoBold | FontKind::MonoBoldItalic | FontKind::SansBold | FontKind::SansBoldItalic | FontKind::SerifBold | FontKind::SerifBoldItalic => {
                    if state.sans_bold_ja.is_none() {
                        state.sans_bold_ja = Some(Font::sans_bold_ja());
                    }
                    return state.sans_bold_ja.as_ref().unwrap();
                },
                FontKind::MarkerFelt => {
                    if state.marker_felt.is_none() {
                        state.marker_felt = Some(Font::marker_felt());
                    }
                    return state.marker_felt.as_ref().unwrap();
                }
            }
        }
    }

    state.get_font_by_kind(kind)
}

/// Read a UIFont's rendering properties. Invalid objects are diagnosed rather
/// than substituted: continuing with a fabricated font leaves the invalid
/// receiver in guest state and only moves the failure to the next message.
fn font_properties(env: &Environment, font: id) -> (FontKind, CGFloat) {
    let Some(mut host_object) = env.objc.get_host_object(font) else {
        if let Some(deallocated) = env
            .framework_state
            .uikit
            .ui_font
            .deallocated_fonts
            .iter()
            .rev()
            .find(|deallocated| deallocated.object == font)
        {
            panic!(
                "UIFont receiver {font:?} was already deallocated (kind {:?}, size {})",
                deallocated.kind, deallocated.size
            );
        }
        panic!("UIFont receiver {font:?} has no registered host object");
    };
    loop {
        if let Some(font) = host_object.as_any().downcast_ref::<UIFontHostObject>() {
            return (font.kind, font.size);
        }
        let Some(superclass) = host_object.as_superclass() else {
            panic!(
                "Expected UIFont for text rendering at {font:?}, found host object type {:?}",
                host_object.type_name()
            );
        };
        host_object = superclass;
    }
}

/// Called by the `sizeWithFont:` method family on `NSString`.
pub fn size_with_font(
    env: &mut Environment,
    font: id,
    text: &str,
    constrained: Option<(CGSize, UILineBreakMode)>,
) -> CGSize {
    let (kind, size) = font_properties(env, font);

    let font = get_font(&mut env.framework_state.uikit.ui_font, kind, text);

    let wrap = constrained.map(|(size, ui_mode)| (size.width, convert_line_break_mode(ui_mode)));

    let (width, height) = font.calculate_text_size(size, text, wrap);

    CGSize { width, height }
}

/// Determine how the text lines will be rendered given a constraint
pub fn break_lines_with_font<'a>(
    env: &mut Environment,
    font: id,
    text: &'a str,
    constrained: Option<(CGSize, UILineBreakMode)>,
) -> Vec<(f32, &'a str)> {
    let (kind, size) = font_properties(env, font);

    let font = get_font(&mut env.framework_state.uikit.ui_font, kind, text);

    let wrap = constrained.map(|(size, ui_mode)| (size.width, convert_line_break_mode(ui_mode)));

    font.break_lines(size, text, wrap)
}

#[inline(always)]
pub fn draw_font_glyph(
    drawer: &mut CGBitmapContextDrawer,
    raster_glyph: crate::font::RasterGlyph,
    fill_color: (f32, f32, f32, f32),
    clip_x: Option<Range<f32>>,
    clip_y: Option<Range<f32>>,
) {
    let scale = raster_glyph.scale();
    let mut glyph_rect = {
        let (x, y) = raster_glyph.origin();
        let (width, height) = raster_glyph.size();
        CGRect {
            origin: CGPoint { x, y },
            size: CGSize { width, height },
        }
    };
    // The code in font.rs won't and can't clip glyphs hanging over the right
    // and bottom sides of the rect, so it has to be done here. Bear in mind
    // that this must not incorrectly affect the texture co-ordinates, otherwise
    // the glyphs become squashed instead.
    // Note that there isn't clipping for the other sides currently because it
    // doesn't seem to be needed.
    if let Some(clip_x) = clip_x {
        if glyph_rect.origin.x >= clip_x.end {
            return;
        }
        if glyph_rect.origin.x + glyph_rect.size.width > clip_x.end {
            glyph_rect.size.width = clip_x.end - glyph_rect.origin.x;
        }
    }
    if let Some(clip_y) = clip_y {
        if glyph_rect.origin.y >= clip_y.end {
            return;
        }
        if glyph_rect.origin.y + glyph_rect.size.height > clip_y.end {
            glyph_rect.size.height = clip_y.end - glyph_rect.origin.y;
        }
    }

    for ((x, y), (tex_x, tex_y)) in drawer.iter_transformed_pixels(glyph_rect) {
        // TODO: bilinear sampling
        let coverage = raster_glyph.pixel_at((
            (tex_x * glyph_rect.size.width * scale - 0.5).round() as i32,
            (tex_y * glyph_rect.size.height * scale - 0.5).round() as i32,
        ));
        let (r, g, b, a) = fill_color;
        let (r, g, b, a) = (r * coverage, g * coverage, b * coverage, a * coverage);
        drawer.put_pixel((x, y), (r, g, b, a), /* blend: */ true);
    }
}

/// Called by the `drawAtPoint:` method family on `NSString`.
pub fn draw_at_point(
    env: &mut Environment,
    font: id,
    text: &str,
    point: CGPoint,
    width_and_line_break_mode: Option<(CGFloat, UILineBreakMode)>,
) -> CGSize {
    let context = UIGraphicsGetCurrentContext(env);

    let (kind, size) = font_properties(env, font);

    let font = get_font(&mut env.framework_state.uikit.ui_font, kind, text);

    let width_and_line_break_mode =
        width_and_line_break_mode.map(|(width, ui_mode)| (width, convert_line_break_mode(ui_mode)));
    let clip_x = width_and_line_break_mode.map(|(width, _)| point.x..(point.x + width));
    let (width, height) = font.calculate_text_size(size, text, width_and_line_break_mode);

    let mut drawer = CGBitmapContextDrawer::new(&env.objc, &mut env.mem, context);
    let fill_color = drawer.rgb_fill_color();
    let oversample = drawer.transform_scale();

    font.draw(
        size,
        text,
        (point.x, point.y),
        width_and_line_break_mode,
        TextAlignment::Left,
        oversample,
        |raster_glyph| {
            draw_font_glyph(
                &mut drawer,
                raster_glyph,
                fill_color,
                clip_x.clone(),
                /* clip_y: */ None,
            )
        },
    );

    CGSize { width, height }
}

/// Called by the `drawInRect:` method family on `NSString`.
pub fn draw_in_rect(
    env: &mut Environment,
    font: id,
    text: &str,
    rect: CGRect,
    line_break_mode: UILineBreakMode,
    alignment: UITextAlignment,
) -> CGSize {
    let context = UIGraphicsGetCurrentContext(env);

    let text_size = size_with_font(env, font, text, Some((rect.size, line_break_mode)));

    let (kind, size) = font_properties(env, font);

    let font = get_font(&mut env.framework_state.uikit.ui_font, kind, text);

    let mut drawer = CGBitmapContextDrawer::new(&env.objc, &mut env.mem, context);
    let fill_color = drawer.rgb_fill_color();
    let oversample = drawer.transform_scale();

    let (origin_x_offset, alignment) = match alignment {
        UITextAlignmentLeft => (0.0, TextAlignment::Left),
        UITextAlignmentCenter => (rect.size.width / 2.0, TextAlignment::Center),
        UITextAlignmentRight => (rect.size.width, TextAlignment::Right),
        _ => unimplemented!(),
    };

    font.draw(
        size,
        text,
        (rect.origin.x + origin_x_offset, rect.origin.y),
        Some((rect.size.width, convert_line_break_mode(line_break_mode))),
        alignment,
        oversample,
        |raster_glyph| {
            draw_font_glyph(
                &mut drawer,
                raster_glyph,
                fill_color,
                /* clip_x: */ Some(rect.origin.x..(rect.origin.x + rect.size.width)),
                /* clip_y: */ Some(rect.origin.y..(rect.origin.y + rect.size.height)),
            )
        },
    );

    text_size
}

fn get_equivalent_font(system_font: &str) -> Option<FontKind> {
    // Maps every font found in every font family in an iOS 2 Simulator
    match system_font {
        // Font Family: Courier
        "Courier" => None,
        "Courier-BoldOblique" => None,
        "Courier-Oblique" => None,
        "Courier-Bold" => None,
        // Font Family: AppleGothic
        "AppleGothic" => None,
        // Font Family: Arial
        "ArialMT" => Some(FontKind::SansRegular),
        "Arial-BoldMT" => Some(FontKind::SansBold),
        "Arial-BoldItalicMT" => Some(FontKind::SansBoldItalic),
        "Arial-ItalicMT" => Some(FontKind::SansItalic),
        // Font Family: STHeiti TC
        "STHeitiTC-Light" => None,
        "STHeitiTC-Medium" => None,
        // Font Family: Hiragino Kaku Gothic ProN
        "HiraKakuProN-W6" => None,
        "HiraKakuProN-W3" => None,
        // Font Family: Courier New
        "CourierNewPS-BoldMT" => Some(FontKind::MonoRegular),
        "CourierNewPS-ItalicMT" => Some(FontKind::MonoBold),
        "CourierNewPS-BoldItalicMT" => Some(FontKind::MonoBoldItalic),
        "CourierNewPSMT" => Some(FontKind::MonoItalic),
        // Font Family: Zapfino
        "Zapfino" => None,
        // Font Family: Arial Unicode MS
        "ArialUnicodeMS" => None,
        // Font Family: STHeiti SC
        "STHeitiSC-Medium" => None,
        "STHeitiSC-Light" => None,
        // Font Family: American Typewriter
        "AmericanTypewriter" => Some(FontKind::MonoRegular),
        "AmericanTypewriter-Bold" => Some(FontKind::MonoBold),
        // Font Family: Helvetica
        "Helvetica-Oblique" => None,
        "Helvetica-BoldOblique" => None,
        "Helvetica" => None,
        "Helvetica-Bold" => None,
        // Font Family: Marker Felt
        "Marker Felt" => Some(FontKind::MarkerFelt),
        "MarkerFelt-Thin" => Some(FontKind::MarkerFelt),
        "MarkerFelt-Wide" => Some(FontKind::MarkerFelt),
        // Font Family: Helvetica Neue
        "HelveticaNeue" => None,
        "HelveticaNeue-Bold" => None,
        // Font Family: DB LCD Temp
        "DBLCDTempBlack" => None,
        // Font Family: Verdana
        "Verdana-Bold" => None,
        "Verdana-BoldItalic" => None,
        "Verdana" => None,
        "Verdana-Italic" => None,
        // Font Family: Times New Roman
        "TimesNewRomanPSMT" => Some(FontKind::SerifRegular),
        "TimesNewRomanPS-BoldMT" => Some(FontKind::SerifBold),
        "TimesNewRomanPS-BoldItalicMT" => Some(FontKind::SerifBoldItalic),
        "TimesNewRomanPS-ItalicMT" => Some(FontKind::SerifItalic),
        // Font Family: Georgia
        "Georgia-Bold" => None,
        "Georgia" => None,
        "Georgia-BoldItalic" => None,
        "Georgia-Italic" => None,
        _ => None,
    }
}
