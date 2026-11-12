use super::*;

pub(super) const B4: i32 = 71;
pub(super) const B5: i32 = 83;
pub(super) const CS6: i32 = 85;
pub(super) const E6: i32 = 88;

pub(super) const F4: i32 = 65;
pub(super) const G4: i32 = 67;
pub(super) const A4: i32 = 69;
pub(super) const BB4: i32 = 70;
pub(super) const C5: i32 = 72;
pub(super) const CS5: i32 = 73;
pub(super) const D5: i32 = 74;
pub(super) const EB5: i32 = 75;
pub(super) const E5: i32 = 76;
pub(super) const F5: i32 = 77;
pub(super) const FS5: i32 = 78;
pub(super) const G5: i32 = 79;
pub(super) const A5: i32 = 81;
pub(super) const BB5: i32 = 82;
pub(super) const C6: i32 = 84;
pub(super) const D6: i32 = 86;

pub(super) type Line = &'static [(i32, f32)];

#[derive(Debug, Clone, Copy)]
pub(super) struct Chord {
    pub(super) root: i32,
    pub(super) minor: bool,
    pub(super) beats: f32,
}

impl Chord {
    pub(super) const fn new(root: i32, minor: bool, beats: f32) -> Self {
        Self { root, minor, beats }
    }

    pub(super) fn third(self) -> i32 {
        if self.minor {
            3
        } else {
            4
        }
    }

    pub(super) fn tones(self) -> [i32; 3] {
        [self.root, self.root + self.third(), self.root + 7]
    }
}

pub(super) const BATTLE_A_CHORDS: &[Chord] = &[
    Chord::new(43, true, 8.0),
    Chord::new(39, false, 8.0),
    Chord::new(46, false, 4.0),
    Chord::new(41, false, 4.0),
    Chord::new(36, true, 4.0),
    Chord::new(38, false, 4.0),
];

pub(super) const BATTLE_B_CHORDS: &[Chord] = &[
    Chord::new(43, true, 4.0),
    Chord::new(39, false, 4.0),
    Chord::new(46, false, 4.0),
    Chord::new(41, false, 4.0),
    Chord::new(36, true, 4.0),
    Chord::new(43, true, 4.0),
    Chord::new(38, false, 8.0),
];

pub(super) const BATTLE_MELODY_A: Line = &[
    (D5, 1.0),
    (G5, 2.0),
    (F5, 1.0),
    (D5, 1.0),
    (BB4, 1.0),
    (C5, 1.0),
    (D5, 1.0),
    (EB5, 1.5),
    (D5, 0.5),
    (C5, 1.0),
    (BB4, 1.0),
    (G4, 1.0),
    (BB4, 3.0),
    (D5, 1.0),
    (F5, 2.0),
    (D5, 1.0),
    (C5, 1.0),
    (F5, 2.0),
    (A4, 1.0),
    (EB5, 1.0),
    (G5, 2.0),
    (EB5, 1.0),
    (FS5, 1.0),
    (A5, 1.0),
    (D6, 1.5),
    (REST, 0.5),
];

pub(super) const BATTLE_MELODY_B: Line = &[
    (G5, 1.0),
    (BB5, 1.5),
    (A5, 0.5),
    (G5, 1.0),
    (EB5, 1.0),
    (G5, 1.5),
    (F5, 0.5),
    (EB5, 1.0),
    (D5, 1.0),
    (F5, 1.5),
    (D5, 0.5),
    (BB4, 1.0),
    (C5, 1.0),
    (F5, 1.5),
    (A5, 0.5),
    (C6, 1.0),
    (C6, 1.0),
    (BB5, 1.0),
    (G5, 1.0),
    (EB5, 1.0),
    (D5, 1.0),
    (BB4, 1.0),
    (G4, 2.0),
    (A4, 1.0),
    (D5, 1.0),
    (FS5, 1.0),
    (A5, 2.0),
    (D6, 3.0),
];

pub(super) const MENU_CHORDS: &[Chord] = &[
    Chord::new(38, true, 8.0),
    Chord::new(46, false, 8.0),
    Chord::new(43, true, 8.0),
    Chord::new(45, false, 8.0),
    Chord::new(38, true, 8.0),
    Chord::new(46, false, 8.0),
    Chord::new(41, false, 8.0),
    Chord::new(45, false, 8.0),
];

pub(super) const MENU_MELODY: Line = &[
    (A4, 3.0),
    (D5, 2.0),
    (F5, 3.0),
    (D5, 2.0),
    (F5, 3.0),
    (BB4, 3.0),
    (BB4, 3.0),
    (D5, 2.0),
    (G5, 3.0),
    (E5, 3.0),
    (CS5, 2.0),
    (A4, 3.0),
    (D5, 3.0),
    (F5, 2.0),
    (A5, 3.0),
    (G5, 3.0),
    (F5, 2.0),
    (D5, 3.0),
    (C5, 3.0),
    (A4, 2.0),
    (F4, 3.0),
    (E5, 4.0),
    (A4, 4.0),
];

pub(super) const INTRO_FANFARE_CHORDS: &[Chord] = &[
    Chord::new(38, false, 4.0),
    Chord::new(47, true, 4.0),
    Chord::new(43, false, 4.0),
    Chord::new(45, false, 4.0),
    Chord::new(38, false, 4.0),
    Chord::new(43, false, 4.0),
    Chord::new(45, false, 4.0),
    Chord::new(38, false, 4.0),
];

pub(super) const INTRO_FANFARE_LINE: Line = &[
    (D5, 1.5),
    (FS5, 0.5),
    (A5, 2.0),
    (B5, 1.5),
    (A5, 0.5),
    (FS5, 1.0),
    (D5, 1.0),
    (G5, 1.5),
    (FS5, 0.5),
    (E5, 1.0),
    (D5, 1.0),
    (E5, 1.0),
    (A5, 1.5),
    (G5, 0.5),
    (E5, 1.0),
    (FS5, 1.5),
    (A5, 0.5),
    (D6, 2.0),
    (B5, 1.5),
    (D6, 0.5),
    (B5, 1.0),
    (G5, 1.0),
    (E5, 1.0),
    (G5, 1.0),
    (A5, 1.0),
    (CS6, 1.0),
    (D6, 4.0),
];

pub(super) const INTRO_CRAWL_CHORDS: &[Chord] = &[
    Chord::new(47, true, 8.0),
    Chord::new(43, false, 8.0),
    Chord::new(38, false, 8.0),
    Chord::new(45, false, 8.0),
];

pub(super) const INTRO_CRAWL_LINE: Line = &[
    (FS5, 3.0),
    (D5, 1.0),
    (B4, 2.0),
    (D5, 2.0),
    (G5, 3.0),
    (B5, 2.0),
    (A5, 1.0),
    (G5, 2.0),
    (FS5, 3.0),
    (A5, 2.0),
    (FS5, 1.0),
    (D5, 2.0),
    (E5, 3.0),
    (CS5, 1.0),
    (E5, 2.0),
    (A4, 2.0),
];

pub(super) const INTRO_FINALE_CHORDS: &[Chord] =
    &[Chord::new(43, false, 4.0), Chord::new(45, false, 4.75)];

pub(super) const INTRO_FINALE_LINE: Line = &[(B5, 2.0), (G5, 2.0), (CS6, 2.0), (E6, 2.75)];
