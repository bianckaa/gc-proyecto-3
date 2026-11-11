use super::*;

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

pub(super) fn battle_mix() -> Mix {
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
