mod assets;
mod decoder;
mod recorder;
mod svg_map;
mod ui;
mod utils;
use assets::Assets;
use crossbeam::channel::{bounded, Receiver, Sender};
use gpui::*;
use kittyaudio::{include_sound, Change, Command, Easing, Mixer, PlaybackRate, Sound};
use std::thread::{self, sleep};
use std::time::Duration;
use ui::{get_window_options, Help, Loop, Root, WaveformModel};
use utils::scale_value;

const SMALLEST_LOOP_UPPER_BOUND: f64 = 0.008;

fn main() {
    let (sizes_sender, sizes_receiver): (
        Sender<(
            Option<Pixels>,
            Option<Pixels>,
            Option<Pixels>,
            Option<GlobalPixels>,
        )>,
        Receiver<(
            Option<Pixels>,
            Option<Pixels>,
            Option<Pixels>,
            Option<GlobalPixels>,
        )>,
    ) = bounded(100);

    let (sound_sender, sound_receiver): (Sender<Sound>, Receiver<Sound>) = bounded(1);

    App::new()
        .with_assets(Assets)
        .run(move |cx: &mut AppContext| {
            cx.activate(true);

            thread::spawn(move || {
                let sound = include_sound!("../assets/audio/piano.wav").unwrap();

                let mut mixer = Mixer::new();
                mixer.init();

                let sound_duration = sound.duration().as_secs_f64();
                let mut sound = mixer.play(sound);
                sound.set_loop_enabled(true);

                let mut previous_start_values = 0.0;
                let mut previous_height_values = 0.0;
                let mut previous_width_values = 0.0;
                let previous_window_width = 0.0;

                loop {
                    match sound_receiver.try_recv() {
                        Ok(new_sound) => {
                            sound.pause();
                            sound = mixer.play(new_sound);
                            sound.set_loop_enabled(true);
                        }
                        Err(_) => {}
                    }
                    match sizes_receiver.try_recv() {
                        Ok((
                            received_loop_start,
                            recieved_loop_height,
                            received_loop_width,
                            received_window_width,
                        )) => {
                            let start_value = received_loop_start
                                .unwrap_or(previous_start_values.into())
                                .0 as f64;
                            let width_value = received_loop_width
                                .unwrap_or(previous_width_values.into())
                                .0 as f64;
                            previous_width_values = width_value;
                            let window_width = received_window_width
                                .unwrap_or(previous_window_width.into())
                                .into();
                            let height_value = recieved_loop_height
                                .unwrap_or(previous_height_values.into())
                                .0 as f64;

                            if previous_start_values != start_value && start_value != 0.0
                                || previous_window_width != width_value && width_value != 0.0
                                || previous_height_values != height_value && height_value != 0.0
                            {
                                previous_height_values = height_value;
                                let scaled_pitch_value =
                                    scale_value(previous_height_values, (0.0, 1024.0), (0.0, 8.0));
                                let scaled_value = scale_value(
                                    start_value,
                                    (0.0, window_width),
                                    (0.0, sound_duration),
                                );
                                previous_start_values = scaled_value;
                                let lower_bound = scaled_value.max(0.0);
                                let upper_bound = lower_bound
                                    + scale_value(
                                        width_value,
                                        (0.0, window_width),
                                        (0.0, sound_duration),
                                    );
                                let upper_bound =
                                    upper_bound.max(lower_bound + SMALLEST_LOOP_UPPER_BOUND);
                                if scaled_value < sound_duration {
                                    let position_command = Command::new(
                                        Change::Position(lower_bound),
                                        Easing::ExpoOut,
                                        0.0,
                                        0.1,
                                    );
                                    sound.add_command(position_command);

                                    let loop_command = Command::new(
                                        Change::LoopSeconds(lower_bound..=upper_bound),
                                        Easing::ExpoOut,
                                        0.0,
                                        0.1,
                                    );
                                    sound.add_command(loop_command);

                                    let pitch_command = Command::new(
                                        Change::PlaybackRate(PlaybackRate::Factor(
                                            scaled_pitch_value,
                                        )),
                                        Easing::ExpoOut,
                                        0.0,
                                        0.1,
                                    );

                                    sound.add_command(pitch_command);
                                }
                            }
                        }
                        Err(_) => {}
                    }
                    sleep(Duration::from_millis(10));
                }
            });

            let sizes_sender = sizes_sender.clone();
            let square: Model<Loop> = cx.new_model(|_| Loop {
                loop_position: 0.0.into(),
                square_height: 128.0.into(),
                square_width: 31.0.into(),
                sender: sizes_sender,
            });

            let help: Model<Help> = cx.new_model(|_| Help {
                text: SharedString::from(""),
                is_shown: false,
            });

            let waveform_model = cx.new_model(|cx| {
                WaveformModel::new(SharedString::from("./assets/audio/piano.wav"), cx)
            });

            cx.open_window(get_window_options(), |cx| {
                cx.new_view(|cx| {
                    Root::new(
                        square,
                        help,
                        waveform_model,
                        cx,
                        sound_sender.clone(),
                    )
                })
            });
        });
}
