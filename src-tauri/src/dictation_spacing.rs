//! Leading space between dictations.
//!
//! Reading the character before the caret is reliable only in apps that expose
//! it through the accessibility API. Terminals, many browser fields, and some
//! Electron editors do not. When the caret cannot be read, a word-like
//! dictation still gets one leading space so two pastes do not run together
//! (`going?Copied`). An empty field in an app that hides the caret can pick up
//! that same leading space; apps that report the caret at position 0 do not.

use log::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaretContext {
    /// Insertion point is at the start of the field, or the whole value is empty.
    AtStart,
    /// The character before the caret is whitespace.
    AfterWhitespace,
    /// The character before the caret is text.
    AfterText,
    /// The focused control did not report a caret. Common in browsers.
    Unknown,
}

/// Punctuation that belongs on the previous word, so it must not gain a space.
fn is_attaching_punctuation(c: char) -> bool {
    matches!(
        c,
        ',' | '.'
            | '!'
            | '?'
            | ';'
            | ':'
            | ')'
            | ']'
            | '}'
            | '%'
            | '，'
            | '。'
            | '！'
            | '？'
            | '；'
            | '：'
            | '）'
            | '】'
            | '」'
            | '』'
            | '、'
            | '…'
    )
}

pub fn apply_leading_space(text: &str) -> String {
    let context = probe_caret_context();
    debug!("Dictation spacing caret context: {context:?}");
    apply_leading_space_with_context(text, context)
}

fn apply_leading_space_with_context(text: &str, context: CaretContext) -> String {
    if text.is_empty() || text.starts_with(char::is_whitespace) {
        return text.to_string();
    }
    if text.chars().next().is_some_and(is_attaching_punctuation) {
        return text.to_string();
    }

    match context {
        CaretContext::AtStart | CaretContext::AfterWhitespace => text.to_string(),
        // Unknown uses the same separator as a known preceding character.
        // Missing a space glues two dictations together; an extra space at the
        // start of a hidden caret is the cheaper failure.
        CaretContext::AfterText | CaretContext::Unknown => format!(" {text}"),
    }
}

fn probe_caret_context() -> CaretContext {
    #[cfg(target_os = "macos")]
    {
        macos::focused_caret_context().unwrap_or(CaretContext::Unknown)
    }
    #[cfg(not(target_os = "macos"))]
    {
        CaretContext::Unknown
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::CaretContext;
    use std::ffi::{c_void, CString};
    use std::ptr;

    type CFTypeRef = *const c_void;
    type AXError = i32;

    const AX_SUCCESS: AXError = 0;
    /// `kAXValueTypeCFRange` from HIServices/AXValue.h.
    const AX_VALUE_CF_RANGE: u32 = 4;
    /// `kCFStringEncodingUTF8`.
    const CF_STRING_UTF8: u32 = 0x0800_0100;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CFRange {
        location: isize,
        length: isize,
    }

    struct OwnedCf(CFTypeRef);

    impl OwnedCf {
        fn new(ptr: CFTypeRef) -> Option<Self> {
            if ptr.is_null() {
                None
            } else {
                Some(Self(ptr))
            }
        }

        fn as_ptr(&self) -> CFTypeRef {
            self.0
        }
    }

    impl Drop for OwnedCf {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { CFRelease(self.0) }
            }
        }
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn CFRelease(cf: CFTypeRef);
        fn CFStringCreateWithCString(
            alloc: *const c_void,
            c_str: *const i8,
            encoding: u32,
        ) -> CFTypeRef;
        fn CFStringGetCString(
            the_string: CFTypeRef,
            buffer: *mut i8,
            buffer_size: isize,
            encoding: u32,
        ) -> u8;
        fn CFGetTypeID(cf: CFTypeRef) -> usize;
        fn CFStringGetTypeID() -> usize;
        fn AXUIElementCreateSystemWide() -> CFTypeRef;
        fn AXUIElementCopyAttributeValue(
            element: CFTypeRef,
            attribute: CFTypeRef,
            value: *mut CFTypeRef,
        ) -> AXError;
        fn AXUIElementCopyParameterizedAttributeValue(
            element: CFTypeRef,
            attribute: CFTypeRef,
            parameter: CFTypeRef,
            result: *mut CFTypeRef,
        ) -> AXError;
        fn AXValueCreate(value_type: u32, value_ptr: *const c_void) -> CFTypeRef;
        fn AXValueGetValue(value: CFTypeRef, value_type: u32, value_ptr: *mut c_void) -> u8;
    }

    fn cf_string(text: &str) -> Option<OwnedCf> {
        let c_string = CString::new(text).ok()?;
        OwnedCf::new(unsafe {
            CFStringCreateWithCString(ptr::null(), c_string.as_ptr(), CF_STRING_UTF8)
        })
    }

    fn copy_attribute(element: CFTypeRef, name: &str) -> Option<OwnedCf> {
        let attribute = cf_string(name)?;
        let mut value: CFTypeRef = ptr::null();
        let status =
            unsafe { AXUIElementCopyAttributeValue(element, attribute.as_ptr(), &mut value) };
        if status != AX_SUCCESS {
            return None;
        }
        OwnedCf::new(value)
    }

    fn cf_string_to_rust(value: CFTypeRef) -> Option<String> {
        if value.is_null() {
            return None;
        }
        unsafe {
            if CFGetTypeID(value) != CFStringGetTypeID() {
                return None;
            }
        }
        let mut buffer = [0i8; 64];
        let ok = unsafe {
            CFStringGetCString(
                value,
                buffer.as_mut_ptr(),
                buffer.len() as isize,
                CF_STRING_UTF8,
            )
        };
        if ok == 0 {
            return None;
        }
        let text = unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }
            .to_str()
            .ok()?;
        Some(text.to_string())
    }

    pub(super) fn focused_caret_context() -> Option<CaretContext> {
        let system = OwnedCf::new(unsafe { AXUIElementCreateSystemWide() })?;
        let focused = copy_attribute(system.as_ptr(), "AXFocusedUIElement")?;

        let Some(range_value) = copy_attribute(focused.as_ptr(), "AXSelectedTextRange") else {
            // No caret range. An empty value still means there is nothing before
            // the insertion point. A non-empty value without a range does not
            // say where the caret is.
            return match copy_attribute(focused.as_ptr(), "AXValue")
                .and_then(|value| cf_string_to_rust(value.as_ptr()))
            {
                Some(value) if value.is_empty() => Some(CaretContext::AtStart),
                _ => None,
            };
        };

        let mut range = CFRange {
            location: 0,
            length: 0,
        };
        let decoded = unsafe {
            AXValueGetValue(
                range_value.as_ptr(),
                AX_VALUE_CF_RANGE,
                &mut range as *mut CFRange as *mut c_void,
            )
        };
        if decoded == 0 || range.location < 0 {
            return None;
        }
        if range.location == 0 {
            return Some(CaretContext::AtStart);
        }

        let previous = CFRange {
            location: range.location - 1,
            length: 1,
        };
        let parameter = OwnedCf::new(unsafe {
            AXValueCreate(
                AX_VALUE_CF_RANGE,
                &previous as *const CFRange as *const c_void,
            )
        })?;
        let attribute = cf_string("AXStringForRange")?;
        let mut text_ref: CFTypeRef = ptr::null();
        let status = unsafe {
            AXUIElementCopyParameterizedAttributeValue(
                focused.as_ptr(),
                attribute.as_ptr(),
                parameter.as_ptr(),
                &mut text_ref,
            )
        };
        if status != AX_SUCCESS {
            // The caret is past the start, which is enough to separate dictations.
            return Some(CaretContext::AfterText);
        }
        let text_value = OwnedCf::new(text_ref)?;
        let Some(previous_text) = cf_string_to_rust(text_value.as_ptr()) else {
            return Some(CaretContext::AfterText);
        };
        if !previous_text.is_empty() && previous_text.chars().all(char::is_whitespace) {
            Some(CaretContext::AfterWhitespace)
        } else {
            Some(CaretContext::AfterText)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_toolkit::text::punctuate_unless_llm;

    #[test]
    fn cleanup_success_skips_the_period_and_both_paths_keep_the_leading_space() {
        let cleaned = punctuate_unless_llm(
            "or not",
            Some("Okay, I'm going to add another sentence now."),
        );
        assert_eq!(cleaned, "Okay, I'm going to add another sentence now.");
        assert_eq!(
            apply_leading_space_with_context(&cleaned, CaretContext::AfterText),
            " Okay, I'm going to add another sentence now."
        );

        // Off, failure, and timeout leave no cleanup text, so the period rule
        // runs and the paste is still separated from the previous dictation.
        let fallback = punctuate_unless_llm("or not", None);
        assert_eq!(fallback, "or not.");
        assert_eq!(
            apply_leading_space_with_context(&fallback, CaretContext::AfterText),
            " or not."
        );
        assert_eq!(
            apply_leading_space_with_context(&fallback, CaretContext::Unknown),
            " or not."
        );
        assert_eq!(
            apply_leading_space_with_context(&fallback, CaretContext::AtStart),
            "or not."
        );
    }

    #[test]
    fn no_space_when_the_field_is_empty() {
        assert_eq!(
            apply_leading_space_with_context("Hello.", CaretContext::AtStart),
            "Hello."
        );
    }

    #[test]
    fn space_when_text_already_precedes_the_caret() {
        assert_eq!(
            apply_leading_space_with_context("Copied text is here.", CaretContext::AfterText),
            " Copied text is here."
        );
    }

    #[test]
    fn no_extra_space_after_whitespace() {
        assert_eq!(
            apply_leading_space_with_context("Hello.", CaretContext::AfterWhitespace),
            "Hello."
        );
    }

    #[test]
    fn hidden_caret_still_separates_a_following_dictation() {
        assert_eq!(
            apply_leading_space_with_context("Copied text is here.", CaretContext::Unknown),
            " Copied text is here."
        );
    }

    #[test]
    fn attaching_punctuation_does_not_gain_a_space() {
        assert_eq!(
            apply_leading_space_with_context(", actually.", CaretContext::AfterText),
            ", actually."
        );
        assert_eq!(
            apply_leading_space_with_context("。", CaretContext::Unknown),
            "。"
        );
    }

    #[test]
    fn existing_leading_whitespace_is_left_alone() {
        assert_eq!(
            apply_leading_space_with_context(" Hello", CaretContext::AfterText),
            " Hello"
        );
    }

    #[test]
    fn empty_text_stays_empty() {
        assert_eq!(
            apply_leading_space_with_context("", CaretContext::AfterText),
            ""
        );
    }
}
