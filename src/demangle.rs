//! Best-effort C++ and Windows symbol demangling.
//!
//! Itanium ABI names (used by GCC/Clang on Linux, macOS, and MinGW) are handled
//! by `cpp_demangle`; MSVC decorated names are handled by `msvc-demangler`.
//! C symbols and symbols that cannot be demangled are left untouched.

use crate::model::FileFormat;
use msvc_demangler::DemangleFlags;

/// Return a readable name when `raw` is a mangled C++ symbol.
///
/// The original name is never discarded by the scanner; callers store the
/// demangled form alongside it.
pub fn demangle(raw: &str, format: FileFormat) -> Option<String> {
    if raw.is_empty() {
        return None;
    }

    // Mach-O prepends an underscore to every external symbol, so a C++ symbol
    // looks like `__ZN...` on disk. Strip exactly one underscore before trying
    // the Itanium demangler.
    let itanium_input = if format == FileFormat::MachO {
        raw.strip_prefix('_').unwrap_or(raw)
    } else {
        raw
    };

    if itanium_input.starts_with("_Z") || itanium_input.starts_with("_R") {
        return demangle_itanium(itanium_input);
    }

    // MSVC C++ names start with `?`.
    if raw.starts_with('?') {
        return demangle_msvc(raw);
    }

    None
}

fn demangle_itanium(input: &str) -> Option<String> {
    let symbol = cpp_demangle::Symbol::new(input.as_bytes()).ok()?;
    let demangled = symbol.demangle().ok()?;
    changed(input, demangled)
}

fn demangle_msvc(input: &str) -> Option<String> {
    let flags = DemangleFlags::COMPLETE
        | DemangleFlags::SPACE_AFTER_COMMA
        | DemangleFlags::SPACE_BEFORE_POINTER
        | DemangleFlags::MS_TYPENAMES
        | DemangleFlags::HUG_TYPE
        | DemangleFlags::DECODE_32_BIT;

    msvc_demangler::demangle(input, flags)
        .ok()
        .and_then(|demangled| changed(input, demangled))
}

fn changed(original: &str, demangled: String) -> Option<String> {
    let demangled = demangled.trim().to_owned();
    if demangled.is_empty() || demangled == original {
        None
    } else {
        Some(demangled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demangles_itanium() {
        let name = "_ZN5space3fooEii";
        assert_eq!(
            demangle(name, FileFormat::Elf).as_deref(),
            Some("space::foo(int, int)")
        );
    }

    #[test]
    fn demangles_itanium_on_macos_with_extra_underscore() {
        let name = "__ZN5space3fooEv";
        assert_eq!(
            demangle(name, FileFormat::MachO).as_deref(),
            Some("space::foo()")
        );
    }

    #[test]
    fn demangles_msvc() {
        let name = "?foo@space@@YAHH@Z";
        let demangled = demangle(name, FileFormat::Pe).expect("MSVC name should demangle");
        assert!(demangled.contains("space::foo"), "{demangled}");
    }

    #[test]
    fn leaves_plain_c_names_alone() {
        assert_eq!(demangle("CreateFileW", FileFormat::Pe), None);
        assert_eq!(demangle("_start", FileFormat::Elf), None);
        assert_eq!(demangle("main", FileFormat::MachO), None);
    }
}
