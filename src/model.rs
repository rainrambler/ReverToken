use serde::Serialize;
use std::fmt;

/// Container format of a scanned binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FileFormat {
    Pe,
    Elf,
    #[serde(rename = "mach-o")]
    MachO,
}

impl FileFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pe => "PE",
            Self::Elf => "ELF",
            Self::MachO => "Mach-O",
        }
    }

    /// A human-readable format name that includes the word size.
    pub fn display_name(self, bits: u8) -> String {
        match self {
            Self::Pe if bits == 64 => "PE32+".to_owned(),
            Self::Pe => "PE32".to_owned(),
            Self::Elf => format!("ELF{bits}"),
            Self::MachO => format!("Mach-O {bits}"),
        }
    }
}

impl fmt::Display for FileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Endianness {
    Little,
    Big,
}

impl Endianness {
    pub fn label(self) -> &'static str {
        match self {
            Self::Little => "LE",
            Self::Big => "BE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    Function,
    Data,
    Unknown,
}

impl SymbolKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Function => "FUNC",
            Self::Data => "DATA",
            Self::Unknown => "SYM",
        }
    }
}

/// One symbol from a PE export directory, an ELF dynamic symbol table, or a
/// Mach-O export trie.
#[derive(Debug, Clone, Serialize)]
pub struct ExportSymbol {
    /// The name exactly as stored in the binary (possibly C++-mangled).
    pub name: String,
    /// The demangled name, when demangling succeeded and changed the name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demangled: Option<String>,
    pub kind: SymbolKind,
    /// Preferred virtual address (PE) or symbol value (ELF/Mach-O).
    pub address: u64,
    /// PE relative virtual address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rva: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// PE export ordinal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordinal: Option<u32>,
    /// PE forwarder target, for example `KERNEL32.Sleep`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forwarder: Option<String>,
}

impl ExportSymbol {
    /// The name to show in the main NAME column: demangled when available,
    /// otherwise the raw symbol name.
    pub fn display_name(&self) -> &str {
        self.demangled.as_deref().unwrap_or(&self.name)
    }

    pub fn matches(&self, query: &SearchQuery) -> bool {
        query.matches(&self.name, self.demangled.as_deref())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BinaryReport {
    pub path: String,
    pub format: FileFormat,
    pub arch: String,
    pub bits: u8,
    pub endianness: Endianness,
    /// DLL, executable, shared library, dylib, ...
    pub kind: String,
    pub exports: Vec<ExportSymbol>,
}

/// Substring filter applied to raw and demangled symbol names.
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    keywords: Vec<String>,
    case_sensitive: bool,
    match_all: bool,
}

impl SearchQuery {
    pub fn new(keywords: Vec<String>, case_sensitive: bool, match_all: bool) -> Self {
        let keywords = keywords
            .into_iter()
            .map(|keyword| {
                if case_sensitive {
                    keyword
                } else {
                    keyword.to_lowercase()
                }
            })
            .collect();

        Self {
            keywords,
            case_sensitive,
            match_all,
        }
    }

    pub fn is_active(&self) -> bool {
        !self.keywords.is_empty()
    }

    pub fn matches(&self, name: &str, demangled: Option<&str>) -> bool {
        if self.keywords.is_empty() {
            return true;
        }

        let name = self.normalize(name);
        let demangled = demangled.map(|value| self.normalize(value));

        let matches_one = |keyword: &str| {
            name.contains(keyword)
                || demangled
                    .as_deref()
                    .is_some_and(|value| value.contains(keyword))
        };

        if self.match_all {
            self.keywords.iter().all(|keyword| matches_one(keyword))
        } else {
            self.keywords.iter().any(|keyword| matches_one(keyword))
        }
    }

    fn normalize<'a>(&self, value: &'a str) -> std::borrow::Cow<'a, str> {
        if self.case_sensitive {
            std::borrow::Cow::Borrowed(value)
        } else {
            std::borrow::Cow::Owned(value.to_lowercase())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_is_case_insensitive_by_default() {
        let query = SearchQuery::new(vec!["createfile".into()], false, false);
        assert!(query.matches("CreateFileW", None));
        assert!(query.matches("_CreateFileW@28", None));
        assert!(!query.matches("CloseHandle", None));
    }

    #[test]
    fn search_checks_demangled_names_too() {
        let query = SearchQuery::new(vec!["foo::bar".into()], false, false);
        assert!(query.matches("_ZN3foo3barEi", Some("foo::bar(int)")));
    }

    #[test]
    fn repeated_keywords_are_or_by_default_and_can_be_and() {
        let either = SearchQuery::new(vec!["foo".into(), "bar".into()], false, false);
        assert!(either.matches("foo", None));
        assert!(either.matches("bar", None));
        assert!(!either.matches("baz", None));

        let all = SearchQuery::new(vec!["foo".into(), "bar".into()], false, true);
        assert!(all.matches("foo::bar", None));
        assert!(!all.matches("foo", None));
    }

    #[test]
    fn case_sensitive_search() {
        let query = SearchQuery::new(vec!["CreateFile".into()], true, false);
        assert!(query.matches("CreateFileW", None));
        assert!(!query.matches("createfilew", None));
    }
}
