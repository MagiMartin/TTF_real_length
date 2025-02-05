use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};
use crate::components::cmap;

pub fn get_cap_height(file: &mut File, cmap_offset: u32, glyf_offset: u32, loca_offset: u32, head_offset: u32) -> io::Result<i16> {
    // Map the character 'H' to its glyph index
    let glyph_index = cmap::parse_cmap(file, cmap_offset, "H")?;

    // Locate the glyph data in the glyf table
    let glyph_offset = get_glyph_offset(file, loca_offset, glyf_offset, glyph_index, head_offset)?;

    // Seek to the glyph data
    file.seek(SeekFrom::Start(glyph_offset as u64))?;

    // Read yMin and yMax
    let _number_of_contours = file.read_i16::<BigEndian>()?; // Number of contours
    let _x_min = file.read_i16::<BigEndian>()?;
    let y_min = file.read_i16::<BigEndian>()?;
    let _x_max = file.read_i16::<BigEndian>()?;
    let y_max = file.read_i16::<BigEndian>()?;

    //Find the glyph offset in the tables and read the ymin and ymax to get the height of 'H'
    //Possible to use OS/2 table to find sChapHeight but it is not always present.

    // Return the capital height
    Ok(y_max - y_min)
}

pub fn get_glyph_offset(file: &mut File, loca_offset: u32, glyf_offset: u32, glyph_index: Vec<u32>, head_offset: u32) -> io::Result<u32> {
    // Seek to the loca table
    file.seek(SeekFrom::Start(loca_offset as u64))?;

    //The index to location ('loca') table stores an array of offsets to the locations of glyph descriptions in the 'glyf' table, 
    //relative to the beginning of that table. Offsets in the array are referenced by corresponding glyph IDs.

    // Determine if the loca table uses 16-bit or 32-bit offsets
    let is_loca_32bit = check_loca_format(file, head_offset)?; // Implement this function to determine loca format

    // Get the glyph offset
    let glyph_offset = if is_loca_32bit {
        // 32-bit offsets
        file.seek(SeekFrom::Start(loca_offset as u64 + (glyph_index[0] as u64 * 4)))?;
        file.read_u32::<BigEndian>()?
    } else {
        // 16-bit offsets (multiplied by 2 to get actual offset)
        file.seek(SeekFrom::Start(loca_offset as u64 + (glyph_index[0] as u64 * 2)))?;
        (file.read_u16::<BigEndian>()? as u32) * 2
    };

    // Return the absolute offset in the glyf table
    Ok(glyf_offset + glyph_offset)
}

pub fn check_loca_format(file: &mut File, head_offset: u32) -> io::Result<bool> {
    // Seek to the head table
    file.seek(SeekFrom::Start(head_offset as u64 + 50))?; // indexToLocFormat is at offset 50 in head table

    // Read indexToLocFormat
    let index_to_loc_format = file.read_u16::<BigEndian>()?;

    // Return true if 32-bit format, false if 16-bit
    Ok(index_to_loc_format == 1)
}
