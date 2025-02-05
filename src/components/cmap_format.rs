use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};

pub fn parse_format0(file: &mut File, input_string: &str) -> io::Result<Vec<u32>> {
    //Continue reading from the position in the cmap table 
    let _length = file.read_u16::<BigEndian>()?; // Length of the subtable
    let _language = file.read_u16::<BigEndian>()?; // Language code
 
    // Read the glyphIdArray (256 bytes)
    let mut glyph_id_array = [0u8; 256];
    file.read_exact(&mut glyph_id_array)?;

    //This is a simple 1 to 1 mapping of character codes to glyph indices
    let mut glyph_ids = Vec::new(); 
    let input_chars: Vec<u16> = input_string.chars().map(|c| c as u16).collect(); 

    for codepoint in input_chars {
        let low_byte = (codepoint & 0xFF) as usize; // Extract the least significant byte
        let glyph_id = glyph_id_array[low_byte] as u32; // Map to a glyph ID
        glyph_ids.push(glyph_id);
    } 
   
    Ok(glyph_ids)
}

pub fn parse_format6(file: &mut File, input_string: &str) -> io::Result<Vec<u32>> {
    //Continue reading from the position in the cmap table 
    let first_code = file.read_u16::<BigEndian>()?;
    let entry_count = file.read_u16::<BigEndian>()?;
    let glyph_id_array: Vec<u16> = (0..entry_count)
        .map(|_| file.read_u16::<BigEndian>().unwrap())
        .collect();

    //Format 6 was designed to map 16-bit characters to glyph indexes when the character codes for a font fall into a single contiguous range.
    let mut glyph_indices = Vec::new();
    for ch in input_string.chars() {
        let codepoint = ch as u16;
        if codepoint >= first_code && codepoint < first_code + entry_count {
            let index = (codepoint - first_code) as usize;
            glyph_indices.push(glyph_id_array[index].into());
        } else {
            println!(
                "Debug: Codepoint U+{:04X} is out of range for cmap format 6",
                codepoint
            );
        }
    }

    Ok(glyph_indices)
}



pub fn parse_format4(file: &mut File, input_string: &str) -> io::Result<Vec<u32>> {
    //Continue reading from the position in the cmap table 
    let _length = file.read_u16::<BigEndian>()?;
    let _language = file.read_u16::<BigEndian>()?;
    let seg_count_x2 = file.read_u16::<BigEndian>()?;
    let seg_count = seg_count_x2 / 2;

    //The format-dependent data is divided into three parts, which must occur in the following order:
    //1. A four-word header gives parameters for an optimized search of the segment list.
    //2. Four parallel arrays describe the segments (one segment for each contiguous range of codes).
    //3. A variable-length array of glyph IDs (unsigned words).
    
    let mut end_counts = Vec::new();
    for _ in 0..seg_count {
        end_counts.push(file.read_u16::<BigEndian>()?);
    }
    let _reserved_pad = file.read_u16::<BigEndian>()?;
    let mut start_counts = Vec::new();
    for _ in 0..seg_count {
        start_counts.push(file.read_u16::<BigEndian>()?);
    }
    let mut id_deltas = Vec::new();
    for _ in 0..seg_count {
        id_deltas.push(file.read_u16::<BigEndian>()?);
    }
    let mut id_range_offsets = Vec::new();
    for _ in 0..seg_count {
        id_range_offsets.push(file.read_u16::<BigEndian>()?);
    }

    let glyph_id_array_offset = file.stream_position()?;

    let mut glyph_indices = Vec::new();

    for ch in input_string.chars() {
        let codepoint = ch as u16;
        
        if let Some(glyph_index) = map_character_to_glyph(
            codepoint,
            &end_counts,
            &start_counts,
            &id_deltas,
            &id_range_offsets,
            glyph_id_array_offset,
            file,
        ) {
            println!("Debug: {} Mapped to glyph index: {}", ch, glyph_index);
            glyph_indices.push(glyph_index.try_into().unwrap());
        } else {
            println!("Character '{}' (U+{:04X}) not mapped.", ch, codepoint);
        }
    }

    Ok(glyph_indices)
}

pub fn map_character_to_glyph(
    codepoint: u16,
    end_counts: &[u16],
    start_counts: &[u16],
    id_deltas: &[u16],
    id_range_offsets: &[u16],
    glyph_id_array_offset: u64,
    file: &mut File,
) -> Option<u32> {
    //Each segment is described by a startCode and endCode, along with an idDelta and an idRangeOffset, which are used for mapping the character codes in the segment.
    for (i, (&end, &start)) in end_counts.iter().zip(start_counts).enumerate() {
        if codepoint >= start && codepoint <= end {
            if id_range_offsets[i] == 0 {
                // Use idDelta directly
                return Some(((codepoint as u32).wrapping_add(id_deltas[i] as u32)) % 65536);
            } else {
                // Compute glyph index using idRangeOffset
                let range_offset_pos = glyph_id_array_offset + (i * 2) as u64;
                file.seek(SeekFrom::Start(range_offset_pos)).ok()?;
                let range_offset = file.read_u16::<BigEndian>().ok()? as u64;

                // Compute glyph array position
                let glyph_array_pos = glyph_id_array_offset + range_offset + ((codepoint - start) as u64 * 2);
                file.seek(SeekFrom::Start(glyph_array_pos)).ok()?;

                // Read glyph ID
                let glyph_id = file.read_u16::<BigEndian>().ok()? as u32;
                if glyph_id != 0 {
                    return Some((glyph_id.wrapping_add(id_deltas[i] as u32)) % 65536);
                }
                return Some(0); // Glyph ID 0 means missing glyph
            }
        }
    }
    None
}

