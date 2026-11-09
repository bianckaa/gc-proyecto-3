pub const GLYPH_WIDTH: i32 = 5;
pub const GLYPH_HEIGHT: i32 = 7;
pub const CHARACTER_ADVANCE: i32 = 6;

const ACUTE: &[(i32, i32)] = &[(3, -2), (2, -1)];
const TILDE: &[(i32, i32)] = &[(0, -1), (1, -2), (2, -1), (3, -2), (4, -1)];
const NO_MARK: &[(i32, i32)] = &[];

fn rows(character: char) -> Option<[u8; GLYPH_HEIGHT as usize]> {
    let rows = match character {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        '.' => [0, 0, 0, 0, 0, 0b01100, 0b01100],
        ',' => [0, 0, 0, 0, 0b01100, 0b00100, 0b01000],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0, 0b00100],
        '?' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0, 0b00100],
        '-' => [0, 0, 0, 0b11111, 0, 0, 0],
        ':' => [0, 0b01100, 0b01100, 0, 0b01100, 0b01100, 0],
        '\'' => [0b00100, 0b00100, 0b01000, 0, 0, 0, 0],
        ' ' => [0; GLYPH_HEIGHT as usize],
        _ => return None,
    };
    Some(rows)
}

fn decompose(character: char) -> (char, &'static [(i32, i32)]) {
    match character {
        'Á' => ('A', ACUTE),
        'É' => ('E', ACUTE),
        'Í' => ('I', ACUTE),
        'Ó' => ('O', ACUTE),
        'Ú' => ('U', ACUTE),
        'Ñ' => ('N', TILDE),
        other => (other, NO_MARK),
    }
}

pub fn supports(character: char) -> bool {
    rows(decompose(character).0).is_some()
}

pub fn text_width(text: &str) -> i32 {
    let count = text.chars().count() as i32;
    if count == 0 {
        0
    } else {
        count * CHARACTER_ADVANCE - (CHARACTER_ADVANCE - GLYPH_WIDTH)
    }
}

pub fn for_each_cell(text: &str, mut visit: impl FnMut(i32, i32)) {
    for (index, character) in text.chars().enumerate() {
        let left = index as i32 * CHARACTER_ADVANCE;
        let (base, marks) = decompose(character);
        let Some(rows) = rows(base) else {
            continue;
        };
        for (row, bits) in rows.iter().enumerate() {
            for column in 0..GLYPH_WIDTH {
                if bits & (1 << (GLYPH_WIDTH - 1 - column)) != 0 {
                    visit(left + column, row as i32);
                }
            }
        }
        for &(column, row) in marks {
            visit(left + column, row);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell_count(text: &str) -> usize {
        let mut count = 0;
        for_each_cell(text, |_, _| count += 1);
        count
    }

    #[test]
    fn width_counts_characters_without_trailing_gap() {
        assert_eq!(text_width(""), 0);
        assert_eq!(text_width("A"), GLYPH_WIDTH);
        assert_eq!(text_width("AB"), GLYPH_WIDTH + CHARACTER_ADVANCE);
    }

    #[test]
    fn accents_add_marks_above_the_base_letter() {
        assert_eq!(cell_count("Á"), cell_count("A") + ACUTE.len());
        assert_eq!(cell_count("Ñ"), cell_count("N") + TILDE.len());
        let mut above = false;
        for_each_cell("É", |_, row| above |= row < 0);
        assert!(above);
    }

    #[test]
    fn unknown_characters_are_reported_and_left_blank() {
        assert!(supports('Ñ'));
        assert!(!supports('#'));
        assert_eq!(cell_count("#"), 0);
    }
}
