use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};

pub fn parse_hmtx(
    file: &mut File,
    glyph_indices: &[u32],
    num_h_metrics: u16,
    hmtx_offset: u32,
    _hhea_offset: u32,
    maxp_offset: u32,
) -> io::Result<u32> {
   
   //Glyph metrics used for horizontal text layout include glyph advance widths, side bearings and X-direction min and max values (xMin, xMax). 
   //These are derived using a combination of the glyph outline data ('glyf', 'CFF ' or CFF2) and the horizontal metrics table. 
   //The horizontal metrics ('hmtx') table provides glyph advance widths and left side bearings. 

    // Calculate num_glyphs by reading the maxp table
    file.seek(SeekFrom::Start(maxp_offset as u64))?;

    // Skip the version (4 bytes) and read numGlyphs (2 bytes)
    file.seek(SeekFrom::Current(4))?;
    let num_glyphs = file.read_u16::<BigEndian>()?;

    // Seek to the start of the hmtx table
    file.seek(SeekFrom::Start(hmtx_offset as u64))?;

    // Read all hMetrics
    let mut h_metrics = Vec::new();
    for _ in 0..num_h_metrics {
        let advance_width = file.read_u16::<BigEndian>()?;
        let _lsb = file.read_i16::<BigEndian>()?; // We can ignore lsb for now
        h_metrics.push(advance_width);
    }

    // Read all lsb values for glyphs beyond num_of_long_hor_metrics
    let mut lsbs = Vec::new();
    for _ in num_h_metrics..num_glyphs {
        let lsb = file.read_i16::<BigEndian>()?;
        lsbs.push(lsb);
    }

    let mut total_width = 0;
    
    // Map glyph indices to advance widths
    for &glyph_index in glyph_indices {
        let advance_width = if (glyph_index as u16) < num_h_metrics {
            h_metrics[glyph_index as usize]
        } else {
            h_metrics[(num_h_metrics - 1) as usize] // Use the last hMetric's advanceWidth
        };

        // Debugging: Print each glyph's advance width
        println!(
            "Glyph Index: {}, Advance Width: {}",
            glyph_index, advance_width
        );

        total_width += advance_width as u32;    
    }

    Ok(total_width)
}
