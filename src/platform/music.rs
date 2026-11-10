use super::audio::SAMPLE_RATE;
use std::f32::consts::TAU;

const PEAK: f32 = 0.85;
const REST: i32 = i32::MIN;
const SECTION_BEATS: f32 = 32.0;
const SECTION_GAINS: [f32; 4] = [0.45, 0.7, 0.9, 1.0];
const INTRO_FANFARE_START: f32 = 11.25;
const INTRO_FANFARE_BEATS: f32 = 32.0;
const INTRO_CRAWL_BEATS: f32 = 32.0;
const INTRO_FINALE_BEATS: f32 = 8.75;
const INTRO_FILE: &str = "assets/music/intro.ogg";

const B4: i32 = 71;
const B5: i32 = 83;
const CS6: i32 = 85;
const E6: i32 = 88;

const F4: i32 = 65;
const G4: i32 = 67;
const A4: i32 = 69;
const BB4: i32 = 70;
const C5: i32 = 72;
const CS5: i32 = 73;
const D5: i32 = 74;
const EB5: i32 = 75;
const E5: i32 = 76;
const F5: i32 = 77;
const FS5: i32 = 78;
const G5: i32 = 79;
const A5: i32 = 81;
const BB5: i32 = 82;
const C6: i32 = 84;
const D6: i32 = 86;

type Line = &'static [(i32, f32)];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicTrack {
    Menu,
    Battle,
    Intro,
}

impl MusicTrack {
    pub const ALL: [MusicTrack; 3] = [MusicTrack::Menu, MusicTrack::Battle, MusicTrack::Intro];

    pub fn index(self) -> usize {
        match self {
            MusicTrack::Menu => 0,
            MusicTrack::Battle => 1,
            MusicTrack::Intro => 2,
        }
    }

    fn beat(self) -> f32 {
        match self {
            MusicTrack::Menu => 1.0,
            MusicTrack::Battle => 0.5,
            MusicTrack::Intro => 2.0 / 3.0,
        }
    }

    fn beats(self) -> f32 {
        match self {
            MusicTrack::Menu => 64.0,
            MusicTrack::Battle => 128.0,
            MusicTrack::Intro => 84.0,
        }
    }

    pub fn external_file(self) -> Option<&'static str> {
        match self {
            MusicTrack::Intro => Some(INTRO_FILE),
            MusicTrack::Menu | MusicTrack::Battle => None,
        }
    }

    pub fn duration(self) -> f32 {
        self.beats() * self.beat()
    }
}

#[derive(Debug, Clone, Copy)]
struct Chord {
    root: i32,
    minor: bool,
    beats: f32,
}

impl Chord {
    const fn new(root: i32, minor: bool, beats: f32) -> Self {
        Self { root, minor, beats }
    }

    fn third(self) -> i32 {
        if self.minor {
            3
        } else {
            4
        }
    }

    fn tones(self) -> [i32; 3] {
        [self.root, self.root + self.third(), self.root + 7]
    }
}

const BATTLE_A_CHORDS: &[Chord] = &[
    Chord::new(43, true, 8.0),
    Chord::new(39, false, 8.0),
    Chord::new(46, false, 4.0),
    Chord::new(41, false, 4.0),
    Chord::new(36, true, 4.0),
    Chord::new(38, false, 4.0),
];

const BATTLE_B_CHORDS: &[Chord] = &[
    Chord::new(43, true, 4.0),
    Chord::new(39, false, 4.0),
    Chord::new(46, false, 4.0),
    Chord::new(41, false, 4.0),
    Chord::new(36, true, 4.0),
    Chord::new(43, true, 4.0),
    Chord::new(38, false, 8.0),
];

const BATTLE_MELODY_A: Line = &[
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

const BATTLE_MELODY_B: Line = &[
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

const MENU_CHORDS: &[Chord] = &[
    Chord::new(38, true, 8.0),
    Chord::new(46, false, 8.0),
    Chord::new(43, true, 8.0),
    Chord::new(45, false, 8.0),
    Chord::new(38, true, 8.0),
    Chord::new(46, false, 8.0),
    Chord::new(41, false, 8.0),
    Chord::new(45, false, 8.0),
];

const MENU_MELODY: Line = &[
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

const INTRO_FANFARE_CHORDS: &[Chord] = &[
    Chord::new(38, false, 4.0),
    Chord::new(47, true, 4.0),
    Chord::new(43, false, 4.0),
    Chord::new(45, false, 4.0),
    Chord::new(38, false, 4.0),
    Chord::new(43, false, 4.0),
    Chord::new(45, false, 4.0),
    Chord::new(38, false, 4.0),
];

const INTRO_FANFARE_LINE: Line = &[
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

const INTRO_CRAWL_CHORDS: &[Chord] = &[
    Chord::new(47, true, 8.0),
    Chord::new(43, false, 8.0),
    Chord::new(38, false, 8.0),
    Chord::new(45, false, 8.0),
];

const INTRO_CRAWL_LINE: Line = &[
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

const INTRO_FINALE_CHORDS: &[Chord] = &[Chord::new(43, false, 4.0), Chord::new(45, false, 4.75)];

const INTRO_FINALE_LINE: Line = &[(B5, 2.0), (G5, 2.0), (CS6, 2.0), (E6, 2.75)];

struct Tone {
    attack: f32,
    release: f32,
    detunes: &'static [f32],
    dull: f32,
    bright: f32,
    vibrato: f32,
}

const BRASS: Tone = Tone {
    attack: 0.05,
    release: 0.2,
    detunes: &[0.997, 1.0, 1.003],
    dull: 0.1,
    bright: 0.45,
    vibrato: 0.004,
};

const HORN: Tone = Tone {
    attack: 0.35,
    release: 0.5,
    detunes: &[0.998, 1.002],
    dull: 0.08,
    bright: 0.25,
    vibrato: 0.006,
};

const STRINGS: Tone = Tone {
    attack: 0.45,
    release: 0.6,
    detunes: &[0.994, 0.998, 1.002, 1.006],
    dull: 0.06,
    bright: 0.14,
    vibrato: 0.003,
};

const PLUCK: Tone = Tone {
    attack: 0.008,
    release: 0.07,
    detunes: &[0.998, 1.002],
    dull: 0.1,
    bright: 0.3,
    vibrato: 0.0,
};

struct Mix {
    samples: Vec<f32>,
}

impl Mix {
    fn new(seconds: f32) -> Self {
        Self {
            samples: vec![0.0; count(seconds)],
        }
    }

    fn add(&mut self, start_seconds: f32, voice: &[f32], gain: f32) {
        let length = self.samples.len();
        let start = count(start_seconds);
        for (offset, value) in voice.iter().enumerate() {
            self.samples[(start + offset) % length] += value * gain;
        }
    }
}

fn count(seconds: f32) -> usize {
    (seconds * SAMPLE_RATE as f32) as usize
}

fn frequency(note: i32) -> f32 {
    440.0 * 2f32.powf((note - 69) as f32 / 12.0)
}

fn envelope(time: f32, length: f32, attack: f32, release: f32) -> f32 {
    let rise = (time / attack).min(1.0);
    let fall = ((length + release - time) / release).clamp(0.0, 1.0);
    rise * fall
}

fn noise_source(seed: u32) -> impl FnMut() -> f32 {
    let mut state = seed;
    move || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (state >> 8) as f32 / (1u32 << 23) as f32 - 1.0
    }
}

fn tone(note: i32, length: f32, spec: &Tone) -> Vec<f32> {
    let base = frequency(note);
    let rate = SAMPLE_RATE as f32;
    let mut phases = vec![0.0f32; spec.detunes.len()];
    let mut filtered = 0.0;
    let normal = 1.0 / spec.detunes.len() as f32;
    (0..count(length + spec.release))
        .map(|index| {
            let time = index as f32 / rate;
            let wobble = 1.0 + spec.vibrato * (time * 5.5 * TAU).sin() * (time / 0.5).min(1.0);
            let mut raw = 0.0;
            for (phase, detune) in phases.iter_mut().zip(spec.detunes) {
                *phase = (*phase + base * detune * wobble / rate).fract();
                raw += *phase * 2.0 - 1.0;
            }
            let coefficient = spec.dull + (spec.bright - spec.dull) * (time / 0.3).min(1.0);
            filtered += (raw * normal - filtered) * coefficient;
            filtered * envelope(time, length, spec.attack, spec.release)
        })
        .collect()
}

fn bass(note: i32, length: f32) -> Vec<f32> {
    let base = frequency(note);
    let rate = SAMPLE_RATE as f32;
    let release = 0.25;
    let mut phase = 0.0f32;
    (0..count(length + release))
        .map(|index| {
            let time = index as f32 / rate;
            phase = (phase + base / rate).fract();
            let wave = (phase * TAU).sin() + 0.35 * (phase * 2.0 * TAU).sin();
            wave * envelope(time, length, 0.03, release)
        })
        .collect()
}

fn timpani(note: i32, strength: f32) -> Vec<f32> {
    let base = frequency(note);
    let rate = SAMPLE_RATE as f32;
    let mut noise = noise_source(note as u32 + 5);
    let mut phase = 0.0f32;
    (0..count(0.9))
        .map(|index| {
            let time = index as f32 / rate;
            phase = (phase + base * (1.0 + 0.5 * (-time * 30.0).exp()) / rate).fract();
            let body = (phase * TAU).sin() * (-time * 4.5).exp();
            let thump = noise() * (-time * 80.0).exp() * 0.5;
            (body + thump) * strength
        })
        .collect()
}

fn snare() -> Vec<f32> {
    let rate = SAMPLE_RATE as f32;
    let mut noise = noise_source(91);
    (0..count(0.25))
        .map(|index| {
            let time = index as f32 / rate;
            noise() * (-time * 28.0).exp() * 0.6
                + (time * 190.0 * TAU).sin() * (-time * 35.0).exp() * 0.4
        })
        .collect()
}

fn cymbal() -> Vec<f32> {
    let rate = SAMPLE_RATE as f32;
    let mut noise = noise_source(173);
    let mut previous = 0.0;
    (0..count(1.6))
        .map(|index| {
            let time = index as f32 / rate;
            let value = noise();
            let crisp = value - previous;
            previous = value;
            crisp * (-time * 2.2).exp() * 0.5
        })
        .collect()
}

fn bell(note: i32) -> Vec<f32> {
    let base = frequency(note);
    let rate = SAMPLE_RATE as f32;
    let partials = [(1.0, 1.0, 1.2), (2.76, 0.5, 1.8), (5.4, 0.25, 3.0)];
    (0..count(3.0))
        .map(|index| {
            let time = index as f32 / rate;
            partials
                .iter()
                .map(|(ratio, amplitude, decay)| {
                    (time * base * ratio * TAU).sin() * amplitude * (-time * decay).exp()
                })
                .sum::<f32>()
        })
        .collect()
}

fn play_line(mix: &mut Mix, line: Line, start_beat: f32, beat: f32, level: u32) {
    let mut cursor = start_beat;
    for &(note, beats) in line {
        if note != REST {
            let length = beats * beat * 0.92;
            mix.add(cursor * beat, &tone(note, length, &BRASS), 0.32);
            if level >= 2 {
                mix.add(cursor * beat, &tone(note - 12, length, &BRASS), 0.18);
            }
            if level >= 3 {
                mix.add(cursor * beat, &tone(note - 12, length, &STRINGS), 0.12);
            }
        }
        cursor += beats;
    }
}

fn play_solo(mix: &mut Mix, line: Line, start_beat: f32, beat: f32, spec: &Tone, gain: f32) {
    let mut cursor = start_beat;
    for &(note, beats) in line {
        if note != REST {
            mix.add(cursor * beat, &tone(note, beats * beat * 0.95, spec), gain);
        }
        cursor += beats;
    }
}

fn battle_chord(mix: &mut Mix, chord: Chord, start_beat: f32, level: u32) {
    let beat = MusicTrack::Battle.beat();
    let third = chord.third();
    let pattern = [0, 7, 12, 7, 0, 7, third + 12, 7];
    for eighth in 0..(chord.beats * 2.0) as usize {
        let note = chord.root + 12 + pattern[eighth % pattern.len()];
        let start = (start_beat + eighth as f32 * 0.5) * beat;
        mix.add(
            start,
            &tone(note, 0.4 * beat, &PLUCK),
            0.16 + 0.02 * level as f32,
        );
    }
    for position in (0..chord.beats as usize).step_by(2) {
        let start = (start_beat + position as f32) * beat;
        mix.add(start, &bass(chord.root, 1.6 * beat), 0.5);
    }
    if level >= 1 {
        let length = chord.beats * beat;
        let start = start_beat * beat;
        let [root, middle, fifth] = chord.tones();
        mix.add(start, &tone(root + 12, length, &STRINGS), 0.07);
        mix.add(start, &tone(middle + 24, length, &STRINGS), 0.07);
        mix.add(start, &tone(fifth + 24, length, &STRINGS), 0.07);
    }
    if level >= 3 {
        let length = chord.beats * beat;
        let start = start_beat * beat;
        mix.add(start, &tone(chord.root + 12, length, &HORN), 0.1);
        mix.add(start, &tone(chord.root + 19, length, &HORN), 0.1);
    }
}

fn battle_percussion(mix: &mut Mix, section_start: f32, level: u32) {
    let beat = MusicTrack::Battle.beat();
    let drum = 43;
    if level >= 2 {
        mix.add(section_start * beat, &cymbal(), 0.25);
    }
    for bar in 0..8 {
        let bar_start = section_start + bar as f32 * 4.0;
        let strength = if level == 0 { 0.35 } else { 0.6 };
        mix.add(bar_start * beat, &timpani(drum, strength), 0.6);
        mix.add(
            (bar_start + 2.0) * beat,
            &timpani(drum, strength * 0.75),
            0.6,
        );
        if level >= 2 {
            mix.add((bar_start + 1.0) * beat, &timpani(drum, 0.3), 0.6);
            mix.add((bar_start + 3.0) * beat, &timpani(drum, 0.3), 0.6);
            mix.add((bar_start + 1.0) * beat, &snare(), 0.35);
            mix.add((bar_start + 3.0) * beat, &snare(), 0.35);
        }
        if level >= 3 && bar % 2 == 1 {
            for step in 0..3 {
                let start = bar_start + 3.0 + step as f32 / 3.0;
                mix.add(start * beat, &timpani(drum, 0.4), 0.6);
            }
        }
    }
}

fn battle_mix() -> Mix {
    let track = MusicTrack::Battle;
    let mut mix = Mix::new(track.duration());
    let sections: [(&[Chord], Option<Line>, u32); 4] = [
        (BATTLE_A_CHORDS, None, 0),
        (BATTLE_A_CHORDS, Some(BATTLE_MELODY_A), 1),
        (BATTLE_B_CHORDS, Some(BATTLE_MELODY_B), 2),
        (BATTLE_A_CHORDS, Some(BATTLE_MELODY_A), 3),
    ];
    let mut section_start = 0.0;
    for (chords, melody, level) in sections {
        let mut part = Mix::new(track.duration());
        let mut chord_start = section_start;
        for chord in chords {
            battle_chord(&mut part, *chord, chord_start, level);
            chord_start += chord.beats;
        }
        if let Some(line) = melody {
            play_line(&mut part, line, section_start, track.beat(), level);
        }
        battle_percussion(&mut part, section_start, level);
        let gain = SECTION_GAINS[level as usize];
        for (total, sample) in mix.samples.iter_mut().zip(&part.samples) {
            *total += sample * gain;
        }
        section_start += SECTION_BEATS;
    }
    mix
}

fn menu_mix() -> Mix {
    let track = MusicTrack::Menu;
    let beat = track.beat();
    let mut mix = Mix::new(track.duration());
    let pattern = [0, 1, 2, 1, 0, 2, 1, 2];
    let mut chord_start = 0.0;
    for chord in MENU_CHORDS {
        let tones = chord.tones();
        let length = chord.beats * beat;
        let start = chord_start * beat;
        mix.add(start, &bass(chord.root, length), 0.35);
        mix.add(start, &tone(tones[0] + 24, length, &STRINGS), 0.1);
        mix.add(start, &tone(tones[1] + 24, length, &STRINGS), 0.1);
        mix.add(start, &tone(tones[2] + 24, length, &STRINGS), 0.1);
        for step in 0..chord.beats as usize {
            let note = tones[pattern[step % pattern.len()]] + 36;
            mix.add((chord_start + step as f32) * beat, &bell(note), 0.08);
        }
        chord_start += chord.beats;
    }
    play_solo(&mut mix, MENU_MELODY, 0.0, beat, &HORN, 0.3);
    mix
}

fn intro_pad(mix: &mut Mix, chord: Chord, start_beat: f32, gain: f32) {
    let beat = MusicTrack::Intro.beat();
    let length = chord.beats * beat;
    let start = start_beat * beat;
    let [root, middle, fifth] = chord.tones();
    mix.add(start, &tone(root + 12, length, &STRINGS), gain);
    mix.add(start, &tone(middle + 24, length, &STRINGS), gain);
    mix.add(start, &tone(fifth + 24, length, &STRINGS), gain);
}

fn timpani_roll(mix: &mut Mix, start_beat: f32, beats: f32, from: f32, to: f32) {
    let beat = MusicTrack::Intro.beat();
    let steps = (beats * 4.0) as usize;
    for step in 0..steps {
        let progress = step as f32 / steps as f32;
        let strength = from + (to - from) * progress;
        mix.add(
            (start_beat + step as f32 * 0.25) * beat,
            &timpani(38, strength),
            0.6,
        );
    }
}

fn intro_mix() -> Mix {
    let track = MusicTrack::Intro;
    let beat = track.beat();
    let mut mix = Mix::new(track.duration());

    let prelude = INTRO_FANFARE_START * beat;
    mix.add(0.0, &bass(38, prelude), 0.1);
    mix.add(0.0, &tone(50, prelude, &STRINGS), 0.04);
    mix.add(0.0, &tone(57, prelude, &STRINGS), 0.03);

    let mut cursor = INTRO_FANFARE_START;
    mix.add(cursor * beat, &cymbal(), 0.4);
    for (index, chord) in INTRO_FANFARE_CHORDS.iter().enumerate() {
        let start = cursor * beat;
        let length = chord.beats * beat;
        let [root, middle, fifth] = chord.tones();
        mix.add(start, &bass(chord.root, length * 0.9), 0.5);
        mix.add(start, &tone(root + 24, length, &BRASS), 0.1);
        mix.add(start, &tone(middle + 24, length, &BRASS), 0.1);
        mix.add(start, &tone(fifth + 24, length, &BRASS), 0.1);
        intro_pad(&mut mix, *chord, cursor, 0.1);
        if index < INTRO_FANFARE_CHORDS.len() - 1 {
            mix.add(start, &timpani(38, 0.7), 0.6);
            mix.add((cursor + 2.0) * beat, &timpani(38, 0.5), 0.6);
        } else {
            mix.add(start, &cymbal(), 0.3);
            timpani_roll(&mut mix, cursor, chord.beats, 0.2, 0.7);
        }
        cursor += chord.beats;
    }
    play_line(&mut mix, INTRO_FANFARE_LINE, INTRO_FANFARE_START, beat, 3);
    mix.add(
        (INTRO_FANFARE_START + INTRO_FANFARE_BEATS) * beat,
        &cymbal(),
        0.45,
    );

    for chord in INTRO_CRAWL_CHORDS {
        mix.add(
            cursor * beat,
            &bass(chord.root, chord.beats * beat * 0.95),
            0.3,
        );
        mix.add(cursor * beat, &timpani(38, 0.25), 0.6);
        intro_pad(&mut mix, *chord, cursor, 0.1);
        cursor += chord.beats;
    }
    let crawl_start = cursor - INTRO_CRAWL_BEATS;
    play_solo(&mut mix, INTRO_CRAWL_LINE, crawl_start, beat, &HORN, 0.3);

    let finale_start = cursor;
    mix.add(cursor * beat, &cymbal(), 0.3);
    for chord in INTRO_FINALE_CHORDS {
        mix.add(
            cursor * beat,
            &bass(chord.root, chord.beats * beat * 0.95),
            0.45,
        );
        let [root, middle, fifth] = chord.tones();
        for note in [root + 24, middle + 24, fifth + 24] {
            mix.add(cursor * beat, &tone(note, chord.beats * beat, &BRASS), 0.1);
        }
        intro_pad(&mut mix, *chord, cursor, 0.12);
        cursor += chord.beats;
    }
    timpani_roll(&mut mix, finale_start, INTRO_FINALE_BEATS, 0.2, 0.6);
    play_line(&mut mix, INTRO_FINALE_LINE, finale_start, beat, 2);
    mix
}

fn reverb(samples: &mut [f32]) {
    let length = samples.len();
    let dry = samples.to_vec();
    let mut wet = vec![0.0f32; length];
    let combs = [(0.137, 0.42), (0.211, 0.34), (0.317, 0.26)];
    for (seconds, feedback) in combs {
        let delay = count(seconds);
        let mut line = dry.clone();
        let mut filtered = 0.0;
        for pass in 0..2 {
            for index in 0..length {
                let source = line[(index + length - delay) % length];
                filtered += (source - filtered) * 0.5;
                let value = dry[index] + feedback * filtered;
                line[index] = value;
                if pass == 1 {
                    wet[index] += value;
                }
            }
        }
    }
    let share = 1.0 / combs.len() as f32;
    for index in 0..length {
        samples[index] = dry[index] * 0.65 + wet[index] * share * 0.5;
    }
}

pub fn synthesize_track(track: MusicTrack) -> Vec<f32> {
    let mut mix = match track {
        MusicTrack::Menu => menu_mix(),
        MusicTrack::Battle => battle_mix(),
        MusicTrack::Intro => intro_mix(),
    };
    reverb(&mut mix.samples);
    let peak = mix
        .samples
        .iter()
        .fold(0.0f32, |peak, sample| peak.max(sample.abs()));
    if peak > f32::EPSILON {
        let scale = PEAK / peak;
        for sample in &mut mix.samples {
            *sample *= scale;
        }
    }
    mix.samples
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_beats(line: Line) -> f32 {
        line.iter().map(|(_, beats)| beats).sum()
    }

    fn chord_beats(chords: &[Chord]) -> f32 {
        chords.iter().map(|chord| chord.beats).sum()
    }

    #[test]
    fn battle_sections_fill_exactly_eight_bars() {
        assert_eq!(chord_beats(BATTLE_A_CHORDS), SECTION_BEATS);
        assert_eq!(chord_beats(BATTLE_B_CHORDS), SECTION_BEATS);
        assert_eq!(line_beats(BATTLE_MELODY_A), SECTION_BEATS);
        assert_eq!(line_beats(BATTLE_MELODY_B), SECTION_BEATS);
        assert_eq!(SECTION_BEATS * 4.0, MusicTrack::Battle.beats());
    }

    #[test]
    fn menu_melody_matches_its_chords_and_track_length() {
        assert_eq!(chord_beats(MENU_CHORDS), MusicTrack::Menu.beats());
        assert_eq!(line_beats(MENU_MELODY), MusicTrack::Menu.beats());
    }

    #[test]
    fn menu_and_battle_loop_for_a_minute_and_intro_covers_the_crawl() {
        assert_eq!(MusicTrack::Menu.duration(), 64.0);
        assert_eq!(MusicTrack::Battle.duration(), 64.0);
        assert!((MusicTrack::Intro.duration() - 56.0).abs() < 0.01);
    }

    #[test]
    fn intro_sections_fill_the_track_exactly() {
        assert_eq!(chord_beats(INTRO_FANFARE_CHORDS), INTRO_FANFARE_BEATS);
        assert_eq!(line_beats(INTRO_FANFARE_LINE), INTRO_FANFARE_BEATS);
        assert_eq!(chord_beats(INTRO_CRAWL_CHORDS), INTRO_CRAWL_BEATS);
        assert_eq!(line_beats(INTRO_CRAWL_LINE), INTRO_CRAWL_BEATS);
        assert_eq!(chord_beats(INTRO_FINALE_CHORDS), INTRO_FINALE_BEATS);
        assert_eq!(line_beats(INTRO_FINALE_LINE), INTRO_FINALE_BEATS);
        let total =
            INTRO_FANFARE_START + INTRO_FANFARE_BEATS + INTRO_CRAWL_BEATS + INTRO_FINALE_BEATS;
        assert_eq!(total, MusicTrack::Intro.beats());
    }

    #[test]
    fn intro_fanfare_lands_when_the_title_appears() {
        let start = INTRO_FANFARE_START * MusicTrack::Intro.beat();
        assert!((start - 7.5).abs() < 0.01);
    }

    #[test]
    fn intro_is_quiet_until_the_fanfare_then_loud() {
        let samples = synthesize_track(MusicTrack::Intro);
        let rate = SAMPLE_RATE as usize;
        let loudness = |from: usize, to: usize| {
            let part = &samples[from * rate..to * rate];
            (part.iter().map(|s| s * s).sum::<f32>() / part.len() as f32).sqrt()
        };
        let prelude = loudness(0, 7);
        let fanfare = loudness(9, 28);
        let crawl = loudness(32, 50);
        assert!(fanfare > prelude * 2.5, "{prelude} {fanfare}");
        assert!(fanfare > crawl, "{fanfare} {crawl}");
    }

    #[test]
    fn every_track_is_audible_finite_and_within_range() {
        for track in MusicTrack::ALL {
            let samples = synthesize_track(track);
            assert_eq!(samples.len(), count(track.duration()), "{:?}", track);
            assert!(
                samples.iter().all(|sample| sample.is_finite()),
                "{:?}",
                track
            );
            let peak = samples
                .iter()
                .fold(0.0f32, |peak, sample| peak.max(sample.abs()));
            assert!(peak > 0.5 && peak <= PEAK + 1e-4, "{:?} {}", track, peak);
            let energy =
                samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32;
            assert!(energy.sqrt() > 0.02, "{:?}", track);
        }
    }

    #[test]
    fn battle_builds_up_from_the_intro_to_the_final_section() {
        let samples = synthesize_track(MusicTrack::Battle);
        let section = samples.len() / 4;
        let loudness =
            |part: &[f32]| (part.iter().map(|s| s * s).sum::<f32>() / part.len() as f32).sqrt();
        let intro = loudness(&samples[..section]);
        let finale = loudness(&samples[section * 3..]);
        assert!(finale > intro * 1.5, "{intro} {finale}");
    }
}
