use anyhow::{Context, Result};
use goblin::mach::MachO;
use goblin::mach::constants::cputype::{
    CPU_TYPE_ARM, CPU_TYPE_ARM64, CPU_TYPE_I386, CPU_TYPE_POWERPC, CPU_TYPE_POWERPC64,
    CPU_TYPE_X86_64,
};
use goblin::mach::exports::ExportInfo;
use goblin::mach::header::{MH_BUNDLE, MH_DYLIB, MH_EXECUTE};

use crate::demangle;
use crate::model::{BinaryReport, Endianness, ExportSymbol, FileFormat, SymbolKind};

pub fn parse(macho: &MachO<'_>, path: &str, demangle_names: bool) -> Result<Vec<BinaryReport>> {
    if macho.is_object_file() {
        return Ok(Vec::new());
    }

    let exports = macho
        .exports()
        .context("failed to read the Mach-O export trie")?;

    let bits = if macho.is_64 { 64 } else { 32 };
    let endianness = if macho.little_endian {
        Endianness::Little
    } else {
        Endianness::Big
    };
    let arch = cpu_name(macho.header.cputype).to_owned();
    let kind = match macho.header.filetype {
        MH_EXECUTE => "executable",
        MH_DYLIB => "dylib",
        MH_BUNDLE => "bundle",
        _ => "Mach-O image",
    }
    .to_owned();

    let mut symbols = Vec::with_capacity(exports.len());

    for export in exports {
        let name = export.name;
        if name.is_empty() {
            continue;
        }

        let (address, forwarder) = match &export.info {
            ExportInfo::Regular { address, .. } => (*address, None),
            ExportInfo::Reexport {
                lib,
                lib_symbol_name,
                ..
            } => {
                let target = match lib_symbol_name {
                    Some(symbol) => format!("{lib}:{symbol}"),
                    None => format!("{lib}:{name}"),
                };
                (0, Some(target))
            }
            ExportInfo::Stub { stub_offset, .. } => (u64::from(*stub_offset), None),
        };

        let demangled = if demangle_names {
            demangle::demangle(&name, FileFormat::MachO)
        } else {
            None
        };

        symbols.push(ExportSymbol {
            name,
            demangled,
            kind: SymbolKind::Unknown,
            address,
            rva: None,
            size: (export.size > 0).then_some(export.size as u64),
            ordinal: None,
            forwarder,
        });
    }

    Ok(vec![BinaryReport {
        path: path.to_owned(),
        format: FileFormat::MachO,
        arch,
        bits,
        endianness,
        kind,
        exports: symbols,
    }])
}

fn cpu_name(cpu: u32) -> &'static str {
    match cpu {
        CPU_TYPE_I386 => "i386",
        CPU_TYPE_X86_64 => "x86_64",
        CPU_TYPE_ARM => "arm",
        CPU_TYPE_ARM64 => "arm64",
        CPU_TYPE_POWERPC => "ppc",
        CPU_TYPE_POWERPC64 => "ppc64",
        _ => "unknown",
    }
}
