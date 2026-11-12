use super::*;

pub(super) fn menu_mix() -> Mix {
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
