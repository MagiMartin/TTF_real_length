use ttf_parser::{Face, GlyphId};
use clap::{Parser, Subcommand};


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



fn main() {

    let mut input_data: Variables = Variables::default();
    let args = Args::parse();

    match args.cmd {
        Commands::Set{font, capital, text} => {
                input_data.font_input = font;
                input_data.capital_input = capital;
                input_data.text_input = text;
        }
    }

    // Read The font file from commandline arguments
    let font_data = std::fs::read(input_data.font_input).expect("Unable to read font file");
    // Parse the font face (bold, italix regular...), in this program we are using Regular
    let face = Face::parse(&font_data, 0).expect("Failed to parse font");
    // The text to measure from the argument
    let text = input_data.text_input;
    // Set the capital size in millimeters in f32 (e.g 10.0 mm)
    let cap_size_mm = input_data.capital_input;
    // Get units per EM (UPEM), Font units are defined based on UPEM which we get from the head
    // table in the file
    let units_per_em = face.units_per_em();


    // Estimate capital height using the bounding box of the glyph for 'H'
    let cap_height = face.glyph_index('H')
        .and_then(|glyph_id| face.glyph_bounding_box(glyph_id))
        .map(|bbox| bbox.y_max - bbox.y_min)
        .expect("Unable to determine capital height");

    // Calculate scale factor to convert font units to millimeters Scale Factor = (Capital Height in mm * 72) / (sCapital Height in font units * 25,4)
    // the 72 is points per inch and the 25,4 mm = 1 inch, so 1 point = 25,4/72 millimetes
    // IN points per font unit
    let scale_factor = (cap_size_mm * 72.0) / (cap_height as f32 * 25.4);
    //Font size in points is the scaling factor * units per em
    let font_pts = scale_factor * units_per_em as f32;


    // Get glyph IDs for each character in the text from the cmap table
    let glyphs: Vec<GlyphId> = text
        .chars()
        .filter_map(|c| face.glyph_index(c))
        .collect();

    // Calculate the total width in font units
    let mut total_width_units: i16 = 0;

    for i in 0..glyphs.len() {
        // Add advance width from the hmtx table
        let advance_width = face.glyph_hor_advance(glyphs[i]).unwrap_or(0);
        total_width_units += advance_width as i16;
        
        // Add kerning adjustment if there is a next glyph
        if i + 1 < glyphs.len() {
            let kerning = get_kerning(&face, glyphs[i], glyphs[i + 1]).unwrap_or(0);
            total_width_units += kerning;
        }
    }
    

    // Convert total width to millimeters
    // the 72 is points per inch and the 25,4 mm = 1 inch, so 1 point = 25,4/72 millimetes
    let total_width_mm = (total_width_units as f32 * font_pts * 25.4) / (units_per_em as f32 * 72.0);

    // Output the results
    println!("Capital size set to:        {:.2} mm", cap_size_mm);
    println!("Measured Text:              {}", text);
    println!("Total width in millimeters: {:.2} mm", total_width_mm);
}


// Retrieves the kerning value for a pair of glyphs using the `kern` table.
fn get_kerning(face: &Face, left: GlyphId, right: GlyphId) -> Option<i16> {
    let kern = face.tables().kern?;

    for subtable in kern.subtables {
        if subtable.horizontal {
            if let Some(value) = subtable.glyphs_kerning(left, right) {
                return Some(value);
            }
        }
    }
    
    None
}

