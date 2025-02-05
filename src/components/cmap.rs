use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};
use crate::components::cmap_format;

pub fn parse_cmap(file: &mut File, cmap_offset:u32, input_string: &str) -> io::Result<Vec<u32>> {
    //Seek to cmap table
    file.seek(SeekFrom::Start(cmap_offset.into()))?;
    //Read Metrics
    let _version = file.read_u16::<BigEndian>()?;
    let num_subtables = file.read_u16::<BigEndian>()?;

    let mut encoding_records = Vec::new();
    for _ in 0..num_subtables {
        let platform_id = file.read_u16::<BigEndian>()?;
        let encoding_id = file.read_u16::<BigEndian>()?;
        let subtable_offset = file.read_u32::<BigEndian>()?; 
        encoding_records.push((platform_id, encoding_id, subtable_offset));
    }
    //Filter the encoding records to use platform id 3 -> Windows encoding and encoding 1 ->
    //Unicode BMP
    if let Some((_, _, subtable_offset)) = encoding_records
        .iter()
        .find(|&&(platform_id, encoding_id, _)| platform_id == 3 && encoding_id == 1)
    {
        //Seek to the right subtable offset
        file.seek(SeekFrom::Start((cmap_offset + *subtable_offset).into()))?;
        //Find cmap subtable format:
        //Format 0: Byte encoding table
        //Format 4: Segment mapping to delta values
        //Format 6: Trimmed table mapping
        let format = file.read_u16::<BigEndian>()?;
        
        match format {
            4 => {
                return cmap_format::parse_format4(file, input_string);
            }
            0 => {
                return cmap_format::parse_format0(file, input_string);
            }
            6 => {
                return cmap_format::parse_format6(file, input_string);
            }
            12 => {
                println!("Debug: cmap format 12 is currently unsupported.");
            }
            _ => {
                println!("Debug: Unsupported cmap format: {}", format);
            }
        }
    } else {
        println!("Debug: No compatible cmap subtable found.");  
    }

    Ok(Vec::new())
}
