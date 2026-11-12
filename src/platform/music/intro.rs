use super::*;

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

pub(super) fn intro_mix() -> Mix {
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
