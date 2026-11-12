use super::*;

pub(super) struct Tone {
    attack: f32,
    release: f32,
    detunes: &'static [f32],
    dull: f32,
    bright: f32,
    vibrato: f32,
}

pub(super) const BRASS: Tone = Tone {
    attack: 0.05,
    release: 0.2,
    detunes: &[0.997, 1.0, 1.003],
    dull: 0.1,
    bright: 0.45,
    vibrato: 0.004,
};

pub(super) const HORN: Tone = Tone {
    attack: 0.35,
    release: 0.5,
    detunes: &[0.998, 1.002],
    dull: 0.08,
    bright: 0.25,
    vibrato: 0.006,
};

pub(super) const STRINGS: Tone = Tone {
    attack: 0.45,
    release: 0.6,
    detunes: &[0.994, 0.998, 1.002, 1.006],
    dull: 0.06,
    bright: 0.14,
    vibrato: 0.003,
};

pub(super) const PLUCK: Tone = Tone {
    attack: 0.008,
    release: 0.07,
    detunes: &[0.998, 1.002],
    dull: 0.1,
    bright: 0.3,
    vibrato: 0.0,
};

pub(super) struct Mix {
    pub(super) samples: Vec<f32>,
}

impl Mix {
    pub(super) fn new(seconds: f32) -> Self {
        Self {
            samples: vec![0.0; count(seconds)],
        }
    }

    pub(super) fn add(&mut self, start_seconds: f32, voice: &[f32], gain: f32) {
        let length = self.samples.len();
        let start = count(start_seconds);
        for (offset, value) in voice.iter().enumerate() {
            self.samples[(start + offset) % length] += value * gain;
        }
    }
}

pub(super) fn count(seconds: f32) -> usize {
    (seconds * SAMPLE_RATE as f32) as usize
}

pub(super) fn frequency(note: i32) -> f32 {
    440.0 * 2f32.powf((note - 69) as f32 / 12.0)
}

pub(super) fn envelope(time: f32, length: f32, attack: f32, release: f32) -> f32 {
    let rise = (time / attack).min(1.0);
    let fall = ((length + release - time) / release).clamp(0.0, 1.0);
    rise * fall
}

pub(super) fn noise_source(seed: u32) -> impl FnMut() -> f32 {
    let mut state = seed;
    move || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (state >> 8) as f32 / (1u32 << 23) as f32 - 1.0
    }
}

pub(super) fn tone(note: i32, length: f32, spec: &Tone) -> Vec<f32> {
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

pub(super) fn bass(note: i32, length: f32) -> Vec<f32> {
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

pub(super) fn timpani(note: i32, strength: f32) -> Vec<f32> {
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

pub(super) fn snare() -> Vec<f32> {
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

pub(super) fn cymbal() -> Vec<f32> {
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

pub(super) fn bell(note: i32) -> Vec<f32> {
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

pub(super) fn play_line(mix: &mut Mix, line: Line, start_beat: f32, beat: f32, level: u32) {
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

pub(super) fn play_solo(
    mix: &mut Mix,
    line: Line,
    start_beat: f32,
    beat: f32,
    spec: &Tone,
    gain: f32,
) {
    let mut cursor = start_beat;
    for &(note, beats) in line {
        if note != REST {
            mix.add(cursor * beat, &tone(note, beats * beat * 0.95, spec), gain);
        }
        cursor += beats;
    }
}

pub(super) fn reverb(samples: &mut [f32]) {
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
