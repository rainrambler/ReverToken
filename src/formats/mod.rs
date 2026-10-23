mod elf;
mod macho;
mod pe;

use std::fs::File;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::{Context, Result};
use goblin::mach::fat::FatHeader;
use goblin::mach::{Mach, SingleArch};
use goblin::{Hint, Object};

use crate::model::BinaryReport;

/// Inspect one path. Returns an empty vector when the file is not a PE, ELF,
/// or Mach-O executable/shared library (or has no export table).
pub fn inspect(path: &Path, demangle_names: bool) -> Result<Vec<BinaryReport>> {
    let Some(bytes) = read_candidate(path)? else {
        return Ok(Vec::new());
    };

    let display_path = path.to_string_lossy().into_owned();
    let object =
        Object::parse(&bytes).with_context(|| format!("failed to parse {}", path.display()))?;

    match object {
        Object::PE(pe) => pe::parse(&pe, &display_path, demangle_names),
        Object::Elf(elf) => elf::parse(&elf, &display_path, demangle_names),
        Object::Mach(Mach::Binary(macho)) => macho::parse(&macho, &display_path, demangle_names),
        Object::Mach(Mach::Fat(fat)) => {
            let mut reports = Vec::new();
            for index in 0..fat.narches {
                match fat.get(index) {
                    Ok(SingleArch::MachO(macho)) => {
                        reports.extend(macho::parse(&macho, &display_path, demangle_names)?);
                    }
                    Ok(SingleArch::Archive(_)) => {}
                    Err(error) => {
                        return Err(error).with_context(|| {
                            format!("failed to parse Mach-O slice {index} in {}", path.display())
                        });
                    }
                }
            }
            Ok(reports)
        }
        // COFF objects, Unix archives, TE images, and unknown formats are not
        // executables or dynamically linked libraries that we support.
        _ => Ok(Vec::new()),
    }
}

fn read_candidate(path: &Path) -> Result<Option<Vec<u8>>> {
    let mut file = File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
    let mut head = [0u8; 16];
    match file.read_exact(&mut head) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("cannot read {}", path.display()));
        }
    }

    let supported = match goblin::peek_bytes(&head) {
        Ok(Hint::PE | Hint::Elf(_) | Hint::Mach(_)) => true,
        // Java class files share the 0xCAFEBABE magic. A real universal
        // binary has a small architecture count, while Java's next four bytes
        // are the class file version (45+ in practice).
        Ok(Hint::MachFat(_)) => {
            FatHeader::parse(&head).is_ok_and(|header| (1..=32).contains(&header.nfat_arch))
        }
        _ => false,
    };
    if !supported {
        return Ok(None);
    }

    let mut bytes = Vec::new();
    file.seek(SeekFrom::Start(0))
        .with_context(|| format!("cannot rewind {}", path.display()))?;
    file.read_to_end(&mut bytes)
        .with_context(|| format!("cannot read {}", path.display()))?;
    Ok(Some(bytes))
}
