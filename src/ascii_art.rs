// ASCII art logo encoded as 4-bit nibbles
// Nibble values: 0=white, 1=green, 2=red, 3=yellow, 4=blue, 5=space, 6=newline
// RLE: 7-E = repeat last char 1-8 times, F = extended RLE (next byte is count)
const LOGO_DATA_4BIT: &[u8] = &[
    0x5f, 0x15, 0x19, 0x5f, 0x1d, 0x26, 0x5f, 0x12, 0x1f, 0x09, 0x5f, 0x1a, 0x2a, 0x65, 0xf1, 0x31,
    0xf0, 0xb5, 0xf1, 0x72, 0xb6, 0x5f, 0x16, 0x1f, 0x0b, 0x5f, 0x14, 0x2b, 0x65, 0xf1, 0xa1, 0xf0,
    0xb5, 0xf1, 0x02, 0xb6, 0x5f, 0x1d, 0x1f, 0x0a, 0x5f, 0x0e, 0x2b, 0x65, 0xf2, 0x11, 0xf0, 0xa5,
    0xf0, 0xa2, 0xb6, 0x5f, 0x24, 0x1f, 0x0a, 0x5d, 0x2b, 0x5c, 0x09, 0x65, 0xf2, 0x81, 0xf0, 0xa5,
    0x92, 0xb5, 0x90, 0xd6, 0x5f, 0x2c, 0x1f, 0x0a, 0x2b, 0x0f, 0x0c, 0x65, 0xf2, 0xf1, 0xf0, 0x92,
    0x90, 0xf0, 0x96, 0x5f, 0x32, 0x1f, 0x0a, 0x0b, 0x65, 0xf1, 0x01, 0x15, 0xf2, 0x11, 0xf0, 0x93,
    0x33, 0x65, 0xf1, 0x01, 0xb5, 0xf1, 0xa1, 0xf0, 0xa3, 0xe6, 0x5f, 0x10, 0x1b, 0x5f, 0x16, 0x1f,
    0x0a, 0x55, 0x3f, 0x0d, 0x65, 0xf1, 0x01, 0xb5, 0xf1, 0x21, 0xf0, 0xb5, 0x52, 0xb5, 0x53, 0xf0,
    0xb6, 0x5f, 0x10, 0x1b, 0x5f, 0x0f, 0x1f, 0x0b, 0x5a, 0x2b, 0x5a, 0x3f, 0x0a, 0x64, 0x95, 0xf0,
    0xc1, 0xb5, 0xf0, 0xc1, 0xf0, 0xa5, 0xe2, 0xb5, 0xe3, 0xc6, 0x4d, 0x5e, 0x1b, 0x5e, 0x1f, 0x0a,
    0x5f, 0x0c, 0x2b, 0x5f, 0x0b, 0x39, 0x64, 0xf0, 0xa5, 0xb1, 0xb5, 0xb1, 0xf0, 0xa5, 0xf0, 0xf2,
    0xb6, 0x55, 0x4f, 0x0c, 0x55, 0x1b, 0x55, 0x1f, 0x0b, 0x5f, 0x12, 0x2b, 0x65, 0x94, 0xf0, 0xc1,
    0xf0, 0xf5, 0xf1, 0x62, 0xb6, 0x5e, 0x4d, 0x1f, 0x0c, 0x5f, 0x19, 0x2b, 0x65, 0xf0, 0xc4, 0x91,
    0xf0, 0x95, 0xf1, 0xc2, 0xb6, 0x5f, 0x0a, 0x0b, 0x1c, 0x4a, 0x5f, 0x1c, 0x29, 0x65, 0xc0, 0xf0,
    0x91, 0xb4, 0xd6, 0x59, 0x0f, 0x0c, 0x1b, 0x4f, 0x0b, 0x65, 0x90, 0xf0, 0x95, 0x55, 0x1b, 0x55,
    0x54, 0xf0, 0xb6, 0x5a, 0x0a, 0x5c, 0x1b, 0x5c, 0x4f, 0x0b, 0x65, 0xf1, 0x01, 0xb5, 0xf0, 0xa4,
    0xf0, 0xb6, 0x5f, 0x10, 0x1b, 0x5f, 0x0d, 0x4f, 0x0b, 0x65, 0xf1, 0x01, 0xb5, 0xf0, 0xf4, 0xf0,
    0xc6, 0x5f, 0x10, 0x1b, 0x5f, 0x13, 0x4f, 0x0b, 0x65, 0xf1, 0x01, 0xb5, 0xf1, 0x64, 0xf0, 0xc6,
    0x5f, 0x12, 0x19, 0x5f, 0x1b, 0x4f, 0x0a, 0x65, 0xf1, 0x41, 0x15, 0xf1, 0xe4, 0xb0,
];
const LOGO_NIBBLE_COUNT: usize = 603;

pub fn display_ascii_art() {
    // Decode the embedded 4-bit logo data
    let logo_content = decode_logo_4bit();
    let colored_logo = colorize_ascii(&logo_content);
    print!("{}", colored_logo);

    println!(); // Add blank line after ASCII art
}

fn decode_logo_4bit() -> String {
    let mut result = String::new();
    let mut nibbles = Vec::new();

    // Unpack bytes to nibbles
    for byte in LOGO_DATA_4BIT {
        nibbles.push((byte >> 4) & 0xF);
        nibbles.push(byte & 0xF);
    }

    // Only use actual nibbles
    nibbles.truncate(LOGO_NIBBLE_COUNT);

    let mut i = 0;
    let mut last_char = ' ';

    while i < nibbles.len() {
        let nibble = nibbles[i];

        match nibble {
            0x0 => {
                last_char = '0';
                result.push(last_char);
            } // White
            0x1 => {
                last_char = '2';
                result.push(last_char);
            } // Green
            0x2 => {
                last_char = '3';
                result.push(last_char);
            } // Red
            0x3 => {
                last_char = '4';
                result.push(last_char);
            } // Yellow
            0x4 => {
                last_char = '5';
                result.push(last_char);
            } // Blue
            0x5 => {
                last_char = ' ';
                result.push(last_char);
            } // Space
            0x6 => {
                last_char = '\n';
                result.push(last_char);
            } // Newline
            0x7..=0xE => {
                // RLE: repeat last char
                let count = (nibble - 0x6) as usize;
                for _ in 0..count {
                    result.push(last_char);
                }
            }
            0xF => {
                // Extended RLE
                if i + 2 < nibbles.len() {
                    let count = ((nibbles[i + 1] << 4) | nibbles[i + 2]) as usize;
                    for _ in 0..count {
                        result.push(last_char);
                    }
                    i += 2;
                }
            }
            _ => {}
        }

        i += 1;
    }

    result
}

fn colorize_ascii(content: &str) -> String {
    let mut result = String::new();

    // Using 256-color ANSI codes for better color matching
    // Format: \x1b[38;5;{color_number}m
    const RESET: &str = "\x1b[0m";
    const WHITE: &str = "\x1b[38;5;231m"; // Pure white (closest to #FFFFFF)
    const GREEN: &str = "\x1b[38;5;72m"; // Sea green (closest to #48B38F)
    const RED: &str = "\x1b[38;5;168m"; // Pink red (closest to #D1405C)
    const YELLOW: &str = "\x1b[38;5;221m"; // Light yellow (closest to #FACC68)
    const CYAN: &str = "\x1b[38;5;74m"; // Sky blue (closest to #3193C3)

    for line in content.lines() {
        let mut current_color = "";
        for ch in line.chars() {
            match ch {
                '0' => {
                    if current_color != WHITE {
                        result.push_str(WHITE);
                        current_color = WHITE;
                    }
                    result.push('#');
                }
                '2' => {
                    if current_color != GREEN {
                        result.push_str(GREEN);
                        current_color = GREEN;
                    }
                    result.push('#');
                }
                '3' => {
                    if current_color != RED {
                        result.push_str(RED);
                        current_color = RED;
                    }
                    result.push('#');
                }
                '4' => {
                    if current_color != YELLOW {
                        result.push_str(YELLOW);
                        current_color = YELLOW;
                    }
                    result.push('#');
                }
                '5' => {
                    if current_color != CYAN {
                        result.push_str(CYAN);
                        current_color = CYAN;
                    }
                    result.push('#');
                }
                ' ' => result.push(' '),
                _ => result.push(ch),
            }
        }
        if !current_color.is_empty() {
            result.push_str(RESET);
        }
        result.push('\n');
    }

    result
}
