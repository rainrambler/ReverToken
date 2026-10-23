use anyhow::Result;
use goblin::pe::PE;
use goblin::pe::export::Reexport;
use goblin::pe::header::machine_to_str;
use goblin::pe::section_table::IMAGE_SCN_MEM_EXECUTE;

use crate::demangle;
use crate::model::{BinaryReport, Endianness, ExportSymbol, FileFormat, SymbolKind};

pub fn parse(pe: &PE<'_>, path: &str, demangle_names: bool) -> Result<Vec<BinaryReport>> {
    let bits = if pe.is_64 { 64 } else { 32 };
    let arch = machine_to_str(pe.header.coff_header.machine).to_owned();
    let kind = if pe.is_lib { "DLL" } else { "executable" }.to_owned();

    let ordinal_base = pe
        .export_data
        .as_ref()
        .map(|data| data.export_directory_table.ordinal_base)
        .unwrap_or(0);

    let mut exports = Vec::with_capacity(pe.exports.len());

    for (index, export) in pe.exports.iter().enumerate() {
        let Some(name) = export.name.filter(|name| !name.is_empty()) else {
            continue;
        };

        // goblin walks the export name pointer table in order, just like the
        // ordinal table. Pairing the two by index gives us the PE ordinal.
        let ordinal = pe
            .export_data
            .as_ref()
            .and_then(|data| data.export_ordinal_table.get(index))
            .map(|ordinal_index| ordinal_base.wrapping_add(u32::from(*ordinal_index)));

        let forwarder = export.reexport.as_ref().map(format_forwarder);
        let rva = u64::try_from(export.rva).unwrap_or(u64::MAX);
        let address = if forwarder.is_some() {
            0
        } else {
            pe.image_base.saturating_add(rva)
        };

        let symbol_kind = if forwarder.is_some() {
            SymbolKind::Unknown
        } else {
            section_kind(pe, export.rva)
        };

        let demangled = if demangle_names {
            demangle::demangle(name, FileFormat::Pe)
        } else {
            None
        };

        exports.push(ExportSymbol {
            name: name.to_owned(),
            demangled,
            kind: symbol_kind,
            address,
            rva: Some(rva),
            size: (export.size > 0).then_some(export.size as u64),
            ordinal,
            forwarder,
        });
    }

    Ok(vec![BinaryReport {
        path: path.to_owned(),
        format: FileFormat::Pe,
        arch,
        bits,
        endianness: Endianness::Little,
        kind,
        exports,
    }])
}

fn format_forwarder(reexport: &Reexport<'_>) -> String {
    match reexport {
        Reexport::DLLName { export, lib } => format!("{lib}.{export}"),
        Reexport::DLLOrdinal { ordinal, lib } => format!("{lib}.#{ordinal}"),
    }
}

fn section_kind(pe: &PE<'_>, rva: usize) -> SymbolKind {
    let rva = rva as u64;
    for section in &pe.sections {
        let start = u64::from(section.virtual_address);
        let span = u64::from(section.virtual_size.max(section.size_of_raw_data));
        if rva >= start && rva < start.saturating_add(span) {
            return if section.characteristics & IMAGE_SCN_MEM_EXECUTE != 0 {
                SymbolKind::Function
            } else {
                SymbolKind::Data
            };
        }
    }
    SymbolKind::Unknown
}
