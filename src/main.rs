use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};
use crate::components::{kern, hmtx, cmap, glyf};
use clap::{Parser, Subcommand};

mod components;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Set {
        font: String,
        capital: f32,
        text: String
    }
}

#[derive(Default)]
pub struct Variables {
    font_input: String,
    capital_input: f32,
    text_input: String
}

#[derive(Debug, PartialEq)]
struct TableRecord {
    tag: String,
    checksum: u32,
    offset: u32,
    length: u32,
}

fn main() -> io::Result<()> {
    
    //Parse command line Args
    let mut input_data: Variables = Variables::default();
    let args = Args::parse();

    match args.cmd {
        Commands::Set{font, capital, text} => {
                input_data.font_input = font;
                input_data.capital_input = capital;
                input_data.text_input = text;
        }
    }

    //Read Truetype font file
    let ttf_path = input_data.font_input;
    let mut file = File::open(ttf_path)?;
    //Input string to measure
    let input_string = input_data.text_input;
    //cap size  in mm
    let cap_size = input_data.capital_input;

    //Read The first tables of the font
    let _scaler_type = file.read_u32::<BigEndian>()?;
    let num_tables = file.read_u16::<BigEndian>()?;
    let _search_range = file.read_u16::<BigEndian>()?;
    let _entry_selector = file.read_u16::<BigEndian>()?;
    let _range_shift = file.read_u16::<BigEndian>()?;

    //Make table record of the number of records with length and offset
    let mut tables = Vec::new();
    for _ in 0..num_tables {
        let mut tag_bytes = [0; 4];
        file.read_exact(&mut tag_bytes)?;
        let tag = String::from_utf8_lossy(&tag_bytes).to_string();

        let checksum = file.read_u32::<BigEndian>()?;
        let offset = file.read_u32::<BigEndian>()?;
        let length = file.read_u32::<BigEndian>()?;

        tables.push(TableRecord {
            tag,
            checksum,
            offset,
            length,
        });
    }

    // Find necessary table offsets
    let cmap_offset = find_table_offset("cmap", &tables)?;
    let glyf_offset = find_table_offset("glyf", &tables)?;
    let loca_offset = find_table_offset("loca", &tables)?;
    let head_offset = find_table_offset("head", &tables)?; 
    let hmtx_offset = find_table_offset("hmtx", &tables)?; 
    let maxp_offset = find_table_offset("maxp", &tables)?; 
    let hhea_offset = find_table_offset("hhea", &tables)?; 
    let kern_offset = find_table_offset("kern", &tables)?; 

    //Get the necessary info from the offsets
    let upem: u16 = parse_head(&mut file, head_offset)?; 
    let glyph_indices: Vec<u32> = cmap::parse_cmap(&mut file, cmap_offset, &input_string)?;
    let num_h_metrics: u16 = parse_hhea(&mut file, hhea_offset)?;
    let total_width: u32 = hmtx::parse_hmtx(&mut file, &glyph_indices, num_h_metrics, hmtx_offset, hhea_offset, maxp_offset)?;
    let cap_font_height = glyf::get_cap_height(&mut file, cmap_offset, glyf_offset, loca_offset, head_offset);

    //Debug Print
    println!("UPEM = {:?}", upem);
    println!("Cap height of H: {:?}", cap_font_height);
    
    //Check if kern table is present ad apply if it is found
    let mut kerning: i32 = 0;
    if kern_offset != 0 {
        println!("Kern table found at offset: {}", kern_offset);
        kerning = kern::parse_kern_table(&mut file, &glyph_indices, kern_offset)?;
    }
     
    //Calc the length from cap size with the right conversions to mm
    let total_kerning = total_width as i32 + kerning;
    let scale_factor = (cap_size as f32 * 72.0) / (cap_font_height.unwrap() as f32 * 25.4);
    let font_pts = scale_factor * upem as f32;
    let width_mm = (total_kerning as f32 * font_pts * 25.4) / (upem as f32 * 72.0);
    println!("The text: {} is {}mm wide, with capital size {}mm", input_string, width_mm, cap_size);

    Ok(())
}

fn find_table_offset (table_name: &str, tables: &Vec<TableRecord> ) -> io::Result<u32> {
    //Find table offset from tag
    let offset_name = tables.iter().find(|t| t.tag == table_name);

    if offset_name == None {
        println!("{} Table, is not found in this file", table_name);
        Ok(0)
    } else {
        Ok(offset_name.unwrap().offset.into())
    }
}

fn parse_head(file: &mut File, head_offset: u32) -> io::Result<u16> {
    // Seek to the units per em (UPEM) value (offset 18 bytes into the table)
    file.seek(SeekFrom::Start(head_offset as u64 + 18))?;
    let upem = file.read_u16::<BigEndian>()?;
    Ok(upem)
}

fn parse_hhea(file: &mut File, hhea_offset: u32) -> io::Result<u16> {
    // Seek to the hhea table
    file.seek(SeekFrom::Start(hhea_offset as u64))?;
    // Read the hhea table metrics
    let _version = file.read_u32::<BigEndian>()?;
    let _ascent = file.read_i16::<BigEndian>()?;
    let _descent = file.read_i16::<BigEndian>()?;
    let _line_gap = file.read_i16::<BigEndian>()?;
    let num_h_metrics = file.read_u16::<BigEndian>()?;

    Ok(num_h_metrics)
}



