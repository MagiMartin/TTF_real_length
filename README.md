# TTF_real_length
A script to get the real-life length in mm of a text given a font, capital height in mm and a text

Using clap the command to run is: cargo run set [path/to/font.ttf] [capital height in mm (20.0)] [text to measure]

This script parses a truetype font file given: 
platform id 3 -> Windows encoding, 
and encoding 1 -> Unicode BMP,
from the cmap table

It also handles Cmap format 0, 4 and 6.

It applies kerning to the glyfs if the kern table is present.

Instead of using sCapHeight from the OS/2 table, the script calculates y_min and y_max from the char 'H'
to get capheight.
(a choice because sCapHeight is not always present in the font files)
