use anyhow::Result;
use goblin::elf::Elf;
use goblin::elf::header::{ET_DYN, ET_EXEC};
use goblin::elf::section_header::SHN_UNDEF;
use goblin::elf::sym::{
    STB_GLOBAL, STB_GNU_UNIQUE, STB_WEAK, STT_COMMON, STT_FILE, STT_FUNC, STT_GNU_IFUNC,
    STT_NOTYPE, STT_OBJECT, STT_SECTION, STT_TLS, STV_DEFAULT, STV_EXPORTED, STV_PROTECTED,
    st_bind, st_type, st_visibility,
};

use crate::demangle;
use crate::model::{BinaryReport, Endianness, ExportSymbol, FileFormat, SymbolKind};

pub fn parse(elf: &Elf<'_>, path: &str, demangle_names: bool) -> Result<Vec<BinaryReport>> {
    if !matches!(elf.header.e_type, ET_EXEC | ET_DYN) {
        return Ok(Vec::new());
    }

    let bits = if elf.is_64 { 64 } else { 32 };
    let endianness = if elf.little_endian {
        Endianness::Little
    } else {
        Endianness::Big
    };
    let arch = machine_name(elf.header.e_machine).to_owned();
    let kind = match elf.header.e_type {
        ET_DYN if elf.interpreter.is_some() => "PIE executable",
        ET_DYN => "shared library",
        ET_EXEC => "executable",
        _ => "ELF image",
    }
    .to_owned();

    let mut exports = Vec::new();

    for symbol in elf.dynsyms.iter() {
        if symbol.st_name == 0 || symbol.st_shndx as u32 == SHN_UNDEF {
            continue;
        }

        let bind = st_bind(symbol.st_info);
        if !matches!(bind, STB_GLOBAL | STB_WEAK | STB_GNU_UNIQUE) {
            continue;
        }

        let visibility = st_visibility(symbol.st_other);
        if !matches!(visibility, STV_DEFAULT | STV_PROTECTED | STV_EXPORTED) {
            continue;
        }

        let symbol_type = st_type(symbol.st_info);
        if matches!(symbol_type, STT_FILE | STT_SECTION) {
            continue;
        }

        let Some(name) = elf.dynstrtab.get_at(symbol.st_name) else {
            continue;
        };
        if name.is_empty() {
            continue;
        }

        let kind = match symbol_type {
            STT_FUNC | STT_GNU_IFUNC => SymbolKind::Function,
            STT_OBJECT | STT_COMMON | STT_TLS => SymbolKind::Data,
            STT_NOTYPE => SymbolKind::Unknown,
            _ => SymbolKind::Unknown,
        };

        let demangled = if demangle_names {
            demangle::demangle(name, FileFormat::Elf)
        } else {
            None
        };

        exports.push(ExportSymbol {
            name: name.to_owned(),
            demangled,
            kind,
            address: symbol.st_value,
            rva: None,
            size: (symbol.st_size > 0).then_some(symbol.st_size),
            ordinal: None,
            forwarder: None,
        });
    }

    Ok(vec![BinaryReport {
        path: path.to_owned(),
        format: FileFormat::Elf,
        arch,
        bits,
        endianness,
        kind,
        exports,
    }])
}

fn machine_name(machine: u16) -> &'static str {
    match machine {
        0 => "unknown",
        2 => "sparc",
        3 => "x86",
        8 => "mips",
        20 => "powerpc",
        21 => "powerpc64",
        22 => "s390",
        40 => "arm",
        42 => "superh",
        50 => "ia64",
        62 => "x86_64",
        183 => "aarch64",
        243 => "riscv",
        247 => "bpf",
        258 => "loongarch",
        _ => "unknown",
    }
}
