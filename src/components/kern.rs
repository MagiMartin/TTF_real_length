use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;

pub fn parse_kern_table(file: &mut File, glyph_indices: &[u32], kern_offset: u32) -> io::Result<i32> {

    //The kerning table contains values that control inter-character spacing for the glyphs in a font. 
    //Fonts containing CFF outlines are not supported by the 'kern' table and require use of the GPOS table to provide kerning.

    // Seek to the start of the kern table
    file.seek(SeekFrom::Start(kern_offset as u64))?;

    // Read kern table header
    let _version = file.read_u16::<BigEndian>()?;
    let n_tables = file.read_u16::<BigEndian>()?;

    // Loop through subtables to find a usable one (Format 0)
    let mut kerning_pairs = HashMap::new();
    for _ in 0..n_tables {
        let _version = file.read_u16::<BigEndian>()?;
        let length = file.read_u16::<BigEndian>()?;
        let coverage = file.read_u16::<BigEndian>()?;
        let format = (coverage >> 8) & 0xFF;

        if format == 0 {
            // Parse format 0 subtable only one supported by windows
            let n_pairs = file.read_u16::<BigEndian>()?;
            file.seek(SeekFrom::Current(6))?; // Skip searchRange, entrySelector, rangeShift

            for _ in 0..n_pairs {
                let left = file.read_u16::<BigEndian>()?;
                let right = file.read_u16::<BigEndian>()?;
                let value = file.read_i16::<BigEndian>()?;

                kerning_pairs.insert((left, right), value);
            }
        } else {
            // Skip unsupported subtable formats
            file.seek(SeekFrom::Current((length - 6) as i64))?;
        }
    }

    // Calculate kerning adjustments
    let mut total_kerning = 0;
    for i in 0..glyph_indices.len() - 1 {
        let left = glyph_indices[i] as u16;
        let right = glyph_indices[i + 1] as u16;

        if glyph_indices[i] > u16::MAX as u32 || glyph_indices[i + 1] > u16::MAX as u32 {
            println!(
                "Warning: Glyph index out of range for kerning pair ({}, {})",
                glyph_indices[i], glyph_indices[i + 1]
            );
            continue; // Skip invalid glyph indices
        }

        if let Some(kerning_value) = kerning_pairs.get(&(left, right)) {
            println!(
                "Debug: Kerning pair ({}, {}) has adjustment: {}",
                left, right, kerning_value
            );
            total_kerning += *kerning_value as i32;
        }
    }

    Ok(total_kerning)
}

