use std::sync::Arc;

use crate::{
    decoder::{compress_samples, decode_wav, DecodedSamples},
    svg_map::{Icon, IconName},
    utils::{calculate_spacing, scale_values_to_unit_range},
};
use crossbeam::channel::Sender;
use gpui::{
    div, hsla, overlay, point, prelude::FluentBuilder, px, size, AnchorCorner, Bounds, BoxShadow,
    ExternalPaths, GlobalPixels, InteractiveElement, IntoElement, Length, Model, ModelContext,
    ParentElement, Pixels, Point, Render, ScrollDelta, ScrollWheelEvent, SharedString, Styled,
    TitlebarOptions, ViewContext, WindowBounds, WindowKind, WindowOptions,
};
use kittyaudio::Sound;
use smallvec::smallvec;

const MAX_NUMBER_OF_SAMPLES_SHOWN: i32 = 160;
const WAVEFORM_SAMPLES_PIXELS: f32 = 5.0;

// Colors
const BG: (f32, f32, f32, f32) = (0.0, 0.0, 0.76, 1.0);
const WAVEFORM_UPPER: (f32, f32, f32, f32) = (0.58, 0.87, 0.79, 1.0);
const WAVEFORM_UPPER_SH: (f32, f32, f32, f32) = (0.4, 0.2, 0.74, 0.5);
const WAVEFORM_LOWER: (f32, f32, f32, f32) = (0.61, 0.76, 0.76, 0.4);
const WAVEFORM_LOWER_SH: (f32, f32, f32, f32) = (0.3, 0.2, 0.74, 0.3);
const SQUARE: (f32, f32, f32, f32) = (0.0, 0.45, 0.57, 0.5);
const SQUARE_SH: (f32, f32, f32, f32) = (0.0, 0.85, 0.74, 0.9);
const HELP_BG: (f32, f32, f32, f32) = (0.14, 0.43, 0.83, 0.2);
const HELP_TEXT: (f32, f32, f32, f32) = (0.58, 0.14, 0.55, 0.9);
const HELP_SH: (f32, f32, f32, f32) = (0.575, 0.45, 0.84, 0.4);
const HELP_IC: (f32, f32, f32, f32) = (0.2, 0.244, 0.89, 0.5);

pub struct Root {
    loop_model: Model<Loop>,
    help_model: Model<Help>,
    waveform_model: Model<WaveformModel>,
    sound_sender: Sender<Sound>,
}

impl Root {
    pub fn new(
        loop_model: Model<Loop>,
        help_model: Model<Help>,
        waveform_model: Model<WaveformModel>,
        cx: &mut ViewContext<Self>,
        sound_sender: Sender<Sound>,
    ) -> Self {
        cx.observe(&loop_model, |_, _, cx| cx.notify()).detach();
        cx.observe(&help_model, |_, _, cx| cx.notify()).detach();
        Self {
            loop_model,
            help_model,
            waveform_model,
            sound_sender,
        }
    }
}

pub struct WaveformModel {
    path: SharedString,
    samples: Option<Arc<Vec<f32>>>,
}

impl WaveformModel {
    pub fn new(path: SharedString, cx: &mut ModelContext<Self>) -> Self {
// TODO Make async
        let mut new_samples = Some(Arc::new(vec![0.0]));
        match decode_wav(path.to_string()) {
            Ok(DecodedSamples::F32(samples)) => {
                let compressed_samples = compress_samples(&samples, 160);
                let scaled_samples = Arc::new(scale_values_to_unit_range(compressed_samples));

                new_samples = Some(scaled_samples);
            }
            Ok(DecodedSamples::I16(samples)) => {
                let compressed_samples = compress_samples(&samples, 160);
                let scaled_samples = Arc::new(scale_values_to_unit_range(compressed_samples));
                new_samples = Some(scaled_samples);
            }
            Err(e) => eprintln!("Error decoding WAV file: {:?}", e),
        }
        cx.notify();

        Self {
            path,
            samples: new_samples,
        }
    }

    pub fn update_samples(&mut self, path: SharedString, cx: &mut ModelContext<Self>) {
        self.path = path;
// TODO Make async
        match decode_wav(self.path.to_string()) {
            Ok(DecodedSamples::F32(samples)) => {
                let compressed_samples = compress_samples(&samples, 160);
                let scaled_samples = Arc::new(scale_values_to_unit_range(compressed_samples));

                self.samples = Some(scaled_samples);
            }
            Ok(DecodedSamples::I16(samples)) => {
                let compressed_samples = compress_samples(&samples, 160);
                let scaled_samples = Arc::new(scale_values_to_unit_range(compressed_samples));
                self.samples = Some(scaled_samples);
            }
            Err(e) => eprintln!("Error decoding WAV file: {:?}", e),
        }
        cx.notify()
    }
}

#[derive(Debug, Clone)]
pub struct Help {
    pub text: SharedString,
    pub is_shown: bool,
}

impl Help {
    pub fn show_help(&mut self) {
        self.is_shown = !self.is_shown;
        if self.is_shown {
            self.text =
                SharedString::from("      [ H-SCROLL ]\n       loop position\n[ CTRL+V-SCROLL ]\n         loop size\n  [ ALT+V-SCROLL ]\n    pitch adjustment");
        } else {
            self.text = SharedString::from("");
        }
    }
}

impl Render for Help {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child(self.text.clone())
    }
}

#[derive(Debug)]
pub struct Loop {
    pub loop_position: Pixels,
    pub square_height: Pixels,
    pub square_width: Pixels,
    pub sender: Sender<(
        Option<Pixels>,
        Option<Pixels>,
        Option<Pixels>,
        Option<GlobalPixels>,
    )>,
}

impl Loop {
    pub fn change_loop(
        &mut self,
        cx: &mut ModelContext<Self>,
        loop_position: Option<Pixels>,
        square_height: Pixels,
        square_width: Pixels,
        windows_width: GlobalPixels,
    ) {
        if let Some(x) = loop_position {
            self.loop_position += x;
        } else {
            self.loop_position = self.loop_position;
        }
        self.square_width += square_width;
        self.square_height += square_height;
        let _s = self.sender.send((
            Some(self.loop_position),
            Some(self.square_height.clamp(px(0.0), px(360.0))),
            Some(self.square_width),
            Some(windows_width),
        ));
        cx.notify();
    }
}

impl Render for Root {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let (window_width, window_height) = match cx.window_bounds() {
            WindowBounds::Fixed(bounds) => (bounds.size.width.into(), bounds.size.height.into()),
            _ => (800.0, 800.0),
        };

        let new_spacing = calculate_spacing(
            window_width as f32,
            MAX_NUMBER_OF_SAMPLES_SHOWN,
            WAVEFORM_SAMPLES_PIXELS,
        );

        let loop_model = &self.loop_model.read(cx);
        let waveform_model_samples = &self.waveform_model.read(cx).samples;

        let waveform = div().children(
            waveform_model_samples
                .as_ref()
                .iter()
                .flat_map(|samples_vec| samples_vec.iter().enumerate())
                .map(|(i, &v)| {
                    let x_position = i as f32 * (5.0 + new_spacing);
                    let waveform_box =
                        |offset_x: f32,
                         value_multiplier: f32,
                         colors: (f32, f32, f32, f32),
                         shadow_colors: (f32, f32, f32, f32),
                         anchor_corner: AnchorCorner| {
                            overlay()
                                .position(point(px(x_position + offset_x), 390.0.into()))
                                .anchor(anchor_corner)
                                .child(
                                    div()
                                        .w_1()
                                        .h(Length::Definite(px(v * value_multiplier).into()))
                                        .bg(hsla(colors.0, colors.1, colors.2, colors.3))
                                        .rounded_lg()
                                        .shadow(smallvec![BoxShadow {
                                            color: hsla(
                                                shadow_colors.0,
                                                shadow_colors.1,
                                                shadow_colors.2,
                                                shadow_colors.3
                                            ),
                                            blur_radius: px(2.),
                                            offset: Point::default(),
                                            spread_radius: px(2.)
                                        }]),
                                )
                        };

                    let upper_waveform = waveform_box(
                        0.0,
                        80.0,
                        WAVEFORM_UPPER,
                        WAVEFORM_UPPER_SH,
                        AnchorCorner::BottomRight,
                    );
                    let lower_waveform = waveform_box(
                        -4.0,
                        100.0,
                        WAVEFORM_LOWER,
                        WAVEFORM_LOWER_SH,
                        AnchorCorner::TopLeft,
                    );

                    (upper_waveform, lower_waveform)
                })
                .flat_map(|(upper_waveform, lower_waveform)| {
                    std::iter::once(upper_waveform).chain(std::iter::once(lower_waveform))
                }),
        );

        // Construct main view tree
        div()
            .on_drop(cx.listener(|this, path: &ExternalPaths, _cx| {
                let p = path.paths()[0]
                    .to_str()
                    .unwrap_or("../assets/audio/piano.wav")
                    .to_owned();
                let sound = Sound::from_path(p.clone()).unwrap();
                let shared_path = SharedString::from(p);
                this.waveform_model.update(_cx, |a, cx| {
                    a.update_samples(shared_path.clone(), cx);
                });
                let _s = this.sound_sender.send(sound).unwrap();
            }))
            .size_full()
            .bg(hsla(BG.0, BG.1, BG.2, BG.3))
            .child(waveform)
            // Square controls
            .on_scroll_wheel(
                cx.listener(move |this, s: &ScrollWheelEvent, cx| match s.delta {
                    ScrollDelta::Pixels(p) => {
                        this.loop_model
                            .update(cx, |square, cx| match s.modifiers.control {
                                true => square.change_loop(
                                    cx,
                                    Some(0.0001.into()),
                                    0.0.into(),
                                    p.y,
                                    window_width.into(),
                                ),
                                false => match s.modifiers.command {
                                    true => square.change_loop(
                                        cx,
                                        Some(0.0001.into()),
                                        p.y,
                                        0.0.into(),
                                        window_width.into(),
                                    ),
                                    false => square.change_loop(
                                        cx,
                                        Some(p.x),
                                        0.0.into(),
                                        0.0.into(),
                                        window_width.into(),
                                    ),
                                },
                            })
                    }
                    ScrollDelta::Lines(_) => {}
                }),
            )
            // Square view
            .child(
                overlay()
                    .position(point(loop_model.loop_position, 325.0.into()))
                    .child(
                        div()
                            .w(Length::Definite({
                                let length = loop_model.square_width.max(1.0.into());
                                length.into()
                            }))
                            .h_32()
                            .bg(hsla(
                                SQUARE.0,
                                {
                                    let hue = loop_model.square_height / px(360.0);
                                    hue.abs().max(0.00001)
                                },
                                SQUARE.2,
                                SQUARE.3,
                            ))
                            .shadow(smallvec![BoxShadow {
                                color: hsla(
                                    SQUARE_SH.0,
                                    {
                                        let hue = loop_model.square_height / px(360.0);
                                        hue.abs().max(0.00001)
                                    },
                                    SQUARE_SH.2,
                                    SQUARE_SH.3
                                ),
                                blur_radius: px(11.),
                                offset: Point::default(),
                                spread_radius: px(9.)
                            }])
                            .rounded_md(),
                    ),
            )
            // Help view
            .child(
                overlay()
                    .position(point(
                        px(((window_width - 165.0) / 2.0) as f32),
                        px(window_height as f32 * 0.05),
                    ))
                    .child(
                        div()
                            .w(Length::Definite(px(165.0).into()))
                            .when(self.help_model.read(cx).is_shown, |this| {
                                this.bg(hsla(HELP_BG.0, HELP_BG.1, HELP_BG.2, HELP_BG.3))
                                    .rounded_lg()
                                    .p_2()
                                    .text_color(hsla(
                                        HELP_TEXT.0,
                                        HELP_TEXT.1,
                                        HELP_TEXT.2,
                                        HELP_TEXT.3,
                                    ))
                                    .shadow(smallvec![BoxShadow {
                                        color: hsla(HELP_SH.0, HELP_SH.1, HELP_SH.2, HELP_SH.3),
                                        blur_radius: px(11.),
                                        offset: Point::default(),
                                        spread_radius: px(5.)
                                    }])
                                    .child(format!("{}", self.help_model.read(cx).text))
                            }),
                    ),
            )
            // Help icon
            .child(
                overlay()
                    .position(point(15.0.into(), (window_height - 20.0).into()))
                    .child(
                        div()
                            .child(
                                Icon::new(IconName::QuestionMark)
                                    .w_8()
                                    .h_8()
                                    .text_color(hsla(HELP_IC.0, HELP_IC.1, HELP_IC.2, HELP_IC.3)),
                            )
                            // Help controls
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(|this, _, cx| {
                                    this.help_model.update(cx, |help, _cx| help.show_help());
                                }),
                            ),
                    ),
            )
    }
}

pub fn get_window_options() -> WindowOptions {
    return WindowOptions {
        bounds: WindowBounds::Fixed(Bounds {
            origin: point(GlobalPixels::from(1800.0), GlobalPixels::from(700.0)),
            size: size(800.0.into(), 800.0.into()),
        }),
        titlebar: Some(TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: None,
        }),
        center: true,
        focus: true,
        show: false,
        kind: WindowKind::Normal,
        is_movable: true,
        display_id: None,
    };
}
