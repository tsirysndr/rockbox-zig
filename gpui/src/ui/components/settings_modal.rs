use crate::ui::components::eq_slider::{fraction_to_gain, gain_to_fraction, EqSlider};
use crate::ui::components::seek_bar::SeekBar;
use crate::ui::components::text_input::TextInput;
use crate::ui::components::{EqBandLocal, SettingsModal, SettingsTab};
use crate::ui::theme::Theme;
use gpui::{
    div, px, rgba, App, AppContext, Context, ElementId, Entity, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, Window,
};
use gpui::prelude::FluentBuilder;

pub struct SettingsModalView {
    pub music_dir_input: Entity<TextInput>,
    pub player_name_input: Entity<TextInput>,
}

/// Fetch global settings via gRPC and push them into the `SettingsModal` global.
/// Uses the same mpsc-channel pattern as `fetch_and_update_devices` — proven to
/// work across the Tokio / smol executor boundary.
pub fn load_settings_for_modal(cx: &mut App) {
    {
        let m = cx.global_mut::<SettingsModal>();
        m.loading = true;
        m.loaded = false;
    }

    let rt = cx.global::<crate::state::TokioHandle>().0.clone();

    type Payload = crate::api::v1alpha1::GetGlobalSettingsResponse;
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<Payload, String>>(1);

    rt.spawn(async move {
        let result = crate::client::get_global_settings()
            .await
            .map_err(|e| e.to_string());
        let _ = tx.send(result).await;
    });

    cx.spawn(async move |cx| {
        let result = rx.recv().await;
        let _ = cx.update(|cx| {
            let mut m = cx.global::<SettingsModal>().clone();
            match result {
                Some(Ok(s)) => {
                    let rg = s.replaygain_settings.as_ref().cloned();
                    let mut bands: Vec<EqBandLocal> = s
                        .eq_band_settings
                        .iter()
                        .map(|b| EqBandLocal { cutoff: b.cutoff, q: b.q, gain: b.gain })
                        .collect();
                    let defaults = crate::ui::components::default_eq_bands();
                    while bands.len() < defaults.len() {
                        bands.push(defaults[bands.len()].clone());
                    }
                    m.music_dir = s.music_dir.clone();
                    m.player_name = s.player_name.clone();
                    m.eq_enabled = s.eq_enabled;
                    m.eq_precut = s.eq_precut;
                    m.eq_bands = bands;
                    m.shuffle = s.playlist_shuffle;
                    m.crossfade = s.crossfade;
                    m.crossfade_fade_in_delay = s.crossfade_fade_in_delay;
                    m.crossfade_fade_in_duration = s.crossfade_fade_in_duration;
                    m.crossfade_fade_out_delay = s.crossfade_fade_out_delay;
                    m.crossfade_fade_out_duration = s.crossfade_fade_out_duration;
                    m.crossfade_fade_out_mixmode = s.crossfade_fade_out_mixmode;
                    m.replaygain_type = rg.as_ref().map(|r| r.r#type).unwrap_or(3);
                    m.replaygain_preamp = rg.as_ref().map(|r| r.preamp).unwrap_or(0);
                    m.replaygain_noclip = rg.as_ref().map(|r| r.noclip).unwrap_or(false);
                    m.balance = s.balance;
                    m.bass = s.bass;
                    m.treble = s.treble;
                    m.stereo_width = s.stereo_width;
                    m.channel_config = s.channel_config;
                    m.surround_enabled = s.surround_enabled;
                    m.dithering_enabled = s.dithering_enabled;
                }
                _ => {}
            }
            m.loaded = true;
            m.loading = false;
            cx.set_global(m);
        });
    })
    .detach();
}

impl SettingsModalView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let music_dir_input = cx.new(|cx| TextInput::new("Music library path…", cx));
        let player_name_input = cx.new(|cx| TextInput::new("Player name…", cx));

        let mdi = music_dir_input.clone();
        let pni = player_name_input.clone();

        // Only syncs TextInput entities when settings finish loading; the actual
        // async fetch is triggered from the Settings action handler via
        // load_settings_for_modal(), not here.
        cx.observe_global::<SettingsModal>(move |_view, cx| {
            let m = cx.global::<SettingsModal>();
            if m.open && m.loaded {
                let music_dir = m.music_dir.clone();
                let player_name = m.player_name.clone();
                let _ = m;
                if mdi.read(cx).value != music_dir {
                    mdi.update(cx, |input, cx| {
                        input.value = music_dir;
                        cx.notify();
                    });
                }
                if pni.read(cx).value != player_name {
                    pni.update(cx, |input, cx| {
                        input.value = player_name;
                        cx.notify();
                    });
                }
            }
            cx.notify();
        })
        .detach();

        SettingsModalView { music_dir_input, player_name_input }
    }
}

// ── Labels ────────────────────────────────────────────────────────────────────

fn crossfade_label(v: i32) -> &'static str {
    match v {
        1 => "Automatic Track Change",
        2 => "Manual Track Change",
        3 => "Shuffle",
        4 => "Shuffle or Manual Skip",
        5 => "Always",
        _ => "Off",
    }
}

fn mixmode_label(v: i32) -> &'static str {
    if v == 1 { "Crossfade" } else { "Mix" }
}

fn replaygain_label(v: i32) -> &'static str {
    match v {
        0 => "Track Gain",
        1 => "Album Gain",
        2 => "Track Gain if Shuffling",
        _ => "Off",
    }
}

fn channel_label(v: i32) -> &'static str {
    match v {
        1 => "Reverse Stereo",
        2 => "Karaoke",
        3 => "Mono",
        4 => "Left Only",
        5 => "Right Only",
        _ => "Stereo",
    }
}

// Each helper sends only the one field that changed, matching the web UI pattern.
// Sending all fields at once triggers side effects in the firmware (crossfade reset,
// playlist re-shuffle, etc.) on every drag event.

fn save_eq_bands(
    bands: &[crate::ui::components::EqBandLocal],
) -> crate::api::v1alpha1::SaveSettingsRequest {
    use crate::api::v1alpha1::{EqBandSetting, SaveSettingsRequest};
    SaveSettingsRequest {
        eq_band_settings: bands
            .iter()
            .map(|b| EqBandSetting { cutoff: b.cutoff, q: b.q, gain: b.gain })
            .collect(),
        ..Default::default()
    }
}

fn save_eq_enabled(enabled: bool) -> crate::api::v1alpha1::SaveSettingsRequest {
    crate::api::v1alpha1::SaveSettingsRequest {
        eq_enabled: Some(enabled),
        ..Default::default()
    }
}

fn save_single(
    field: impl FnOnce(&mut crate::api::v1alpha1::SaveSettingsRequest),
) -> crate::api::v1alpha1::SaveSettingsRequest {
    let mut req = crate::api::v1alpha1::SaveSettingsRequest::default();
    field(&mut req);
    req
}

// ── Render ────────────────────────────────────────────────────────────────────

impl Render for SettingsModalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let modal = cx.global::<SettingsModal>().clone();
        let theme = *cx.global::<Theme>();

        // Return empty div when closed — no cx borrows needed inside
        if !modal.open {
            return div().id("settings_modal_root_closed").into_any_element();
        }

        let viewport = window.viewport_size();
        let card_w = px(720.0);
        let card_left = (viewport.width - card_w) * 0.5;
        let card_top = px(44.0);

        let accent = theme.switcher_active;
        let track_color = rgba(0xFFFFFF12);
        let text_secondary = theme.library_header_text;
        let text_primary = theme.library_text;
        let border = theme.border;

        // ── All cx work upfront (no nested closures needed) ──

        // Tab listeners
        let tab_general = cx.listener(|_, _, _, cx| {
            cx.global_mut::<SettingsModal>().active_tab = SettingsTab::General;
            cx.notify();
        });
        let tab_eq = cx.listener(|_, _, _, cx| {
            cx.global_mut::<SettingsModal>().active_tab = SettingsTab::Equalizer;
            cx.notify();
        });
        let tab_playback = cx.listener(|_, _, _, cx| {
            cx.global_mut::<SettingsModal>().active_tab = SettingsTab::Playback;
            cx.notify();
        });
        let tab_sound = cx.listener(|_, _, _, cx| {
            cx.global_mut::<SettingsModal>().active_tab = SettingsTab::Sound;
            cx.notify();
        });

        // Close / cancel listeners
        let close_l = cx.listener(|_, _, _, cx| {
            cx.global_mut::<SettingsModal>().open = false;
            cx.notify();
        });
        let cancel_l = cx.listener(|_, _, _, cx| {
            cx.global_mut::<SettingsModal>().open = false;
            cx.notify();
        });
        let backdrop_l = cx.listener(|_, _, _, cx| {
            cx.global_mut::<SettingsModal>().open = false;
            cx.notify();
        });

        // Save listener
        let mdi_save = self.music_dir_input.clone();
        let pni_save = self.player_name_input.clone();
        let save_l = cx.listener(move |_, _, _, cx| {
            let modal = cx.global::<SettingsModal>().clone();
            let music_dir = mdi_save.read(cx).value.clone();
            let player_name = pni_save.read(cx).value.clone();
            let rt = cx.global::<crate::state::TokioHandle>().0.clone();
            use crate::api::v1alpha1::{EqBandSetting, ReplaygainSettings, SaveSettingsRequest};
            let req = SaveSettingsRequest {
                music_dir: Some(music_dir),
                player_name: Some(player_name),
                playlist_shuffle: Some(modal.shuffle),
                eq_enabled: Some(modal.eq_enabled),
                eq_band_settings: modal
                    .eq_bands
                    .iter()
                    .map(|b| EqBandSetting { cutoff: b.cutoff, q: b.q, gain: b.gain })
                    .collect(),
                crossfade: Some(modal.crossfade),
                fade_in_delay: Some(modal.crossfade_fade_in_delay),
                fade_in_duration: Some(modal.crossfade_fade_in_duration),
                fade_out_delay: Some(modal.crossfade_fade_out_delay),
                fade_out_duration: Some(modal.crossfade_fade_out_duration),
                fade_out_mixmode: Some(modal.crossfade_fade_out_mixmode),
                replaygain_settings: Some(ReplaygainSettings {
                    r#type: modal.replaygain_type,
                    preamp: modal.replaygain_preamp,
                    noclip: modal.replaygain_noclip,
                }),
                balance: Some(modal.balance),
                bass: Some(modal.bass),
                treble: Some(modal.treble),
                stereo_width: Some(modal.stereo_width),
                channel_config: Some(modal.channel_config),
                surround_enabled: Some(modal.surround_enabled),
                ..Default::default()
            };
            rt.spawn(crate::client::save_settings_all(req));
            cx.global_mut::<SettingsModal>().open = false;
            cx.notify();
        });

        // Tab content
        let content = match &modal.active_tab {
            SettingsTab::General => self.render_general_tab(&theme, &modal, cx),
            SettingsTab::Equalizer => self.render_eq_tab(&theme, &modal, cx),
            SettingsTab::Playback => self.render_playback_tab(&theme, &modal, cx),
            SettingsTab::Sound => self.render_sound_tab(&theme, &modal, cx),
        };

        // ── Layout ──
        let active_tab = &modal.active_tab;

        let tab_bar = div()
            .flex()
            .flex_row()
            .gap_1()
            .px_6()
            .pt_5()
            .pb_4()
            .child(tab_item("General", active_tab == &SettingsTab::General, accent, text_secondary, text_primary, track_color, tab_general))
            .child(tab_item("Equalizer", active_tab == &SettingsTab::Equalizer, accent, text_secondary, text_primary, track_color, tab_eq))
            .child(tab_item("Playback", active_tab == &SettingsTab::Playback, accent, text_secondary, text_primary, track_color, tab_playback))
            .child(tab_item("Sound", active_tab == &SettingsTab::Sound, accent, text_secondary, text_primary, track_color, tab_sound));

        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_6()
            .pt_5()
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(text_primary)
                    .child("Settings"),
            )
            .child(
                div()
                    .id("settings_close_btn")
                    .w_8()
                    .h_8()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .text_color(text_secondary)
                    .cursor_pointer()
                    .hover(|d| d.bg(track_color).text_color(text_primary))
                    .on_click(close_l)
                    .child("✕"),
            );

        let footer = div()
            .flex()
            .flex_row()
            .justify_end()
            .gap_3()
            .px_6()
            .py_4()
            .border_t_1()
            .border_color(border)
            .child(
                div()
                    .id("settings_cancel")
                    .px_5()
                    .py_2()
                    .rounded_lg()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(text_secondary)
                    .bg(track_color)
                    .cursor_pointer()
                    .hover(|d| d.bg(rgba(0xFFFFFF1F)))
                    .on_click(cancel_l)
                    .child("Cancel"),
            )
            .child(
                div()
                    .id("settings_save")
                    .px_5()
                    .py_2()
                    .rounded_lg()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(text_primary)
                    .bg(accent)
                    .cursor_pointer()
                    .hover(|d| d.bg(rgba(0x6F00FFD0)))
                    .on_click(save_l)
                    .child("Save"),
            );

        let card = div()
            .id("settings_card")
            .absolute()
            .top(card_top)
            .left(card_left)
            .w(card_w)
            .bg(theme.titlebar_bg)
            .rounded_xl()
            .border_1()
            .border_color(border)
            .occlude()
            .flex()
            .flex_col()
            .child(header)
            .child(tab_bar)
            .child(
                div()
                    .id("settings_content_scroll")
                    .px_6()
                    .pb_4()
                    .max_h(px(500.0))
                    .overflow_y_scroll()
                    .child(content),
            )
            .child(footer);

        let backdrop = div()
            .id("settings_backdrop")
            .absolute()
            .inset_0()
            .bg(rgba(0x000000B0))
            .occlude()
            .on_click(backdrop_l);

        div()
            .id("settings_modal_root")
            .absolute()
            .inset_0()
            .child(backdrop)
            .child(card)
            .into_any_element()
    }
}

fn tab_item(
    label: &'static str,
    is_active: bool,
    accent: gpui::Rgba,
    text_secondary: gpui::Rgba,
    text_primary: gpui::Rgba,
    track_color: gpui::Rgba,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .id(ElementId::Name(label.into()))
        .px_4()
        .py(px(6.0))
        .rounded_lg()
        .text_sm()
        .cursor_pointer()
        .font_weight(gpui::FontWeight::MEDIUM)
        .when(is_active, |d| d.bg(accent).text_color(text_primary))
        .when(!is_active, |d| {
            d.text_color(text_secondary).hover(|d| d.bg(track_color).text_color(text_primary))
        })
        .on_click(on_click)
        .child(label)
}

// ── Tab content ───────────────────────────────────────────────────────────────

impl SettingsModalView {
    fn render_general_tab(
        &self,
        theme: &Theme,
        _modal: &SettingsModal,
        _cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let text_secondary = theme.library_header_text;
        let text_primary = theme.library_text;
        div()
            .flex()
            .flex_col()
            .gap_5()
            .py_2()
            .child(section_header("Library", text_secondary))
            .child(setting_field(
                "Music Library Path",
                "Path scanned for audio files",
                text_primary,
                text_secondary,
                self.music_dir_input.clone(),
            ))
            .child(section_header("Identity", text_secondary))
            .child(setting_field(
                "Player Name",
                "Displayed to other clients on the network",
                text_primary,
                text_secondary,
                self.player_name_input.clone(),
            ))
            .into_any_element()
    }

    fn render_eq_tab(
        &self,
        theme: &Theme,
        modal: &SettingsModal,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let text_secondary = theme.library_header_text;
        let text_primary = theme.library_text;
        let accent = theme.switcher_active;

        let eq_enabled = modal.eq_enabled;
        let eq_precut = modal.eq_precut;
        let precut_frac = (eq_precut as f32 / 240.0).clamp(0.0, 1.0);
        let precut_db = eq_precut as f32 / 10.0;

        let eq_toggle_l = cx.listener(|_, _, _, cx| {
            let v = cx.global::<SettingsModal>().eq_enabled;
            let new_v = !v;
            cx.global_mut::<SettingsModal>().eq_enabled = new_v;
            let rt = cx.global::<crate::state::TokioHandle>().0.clone();
            rt.spawn(crate::client::save_settings_all(save_eq_enabled(new_v)));
            cx.notify();
        });

        // Firmware convention: cutoff = center freq (Hz), q = Q factor, gain = gain (tenths dB)
        // Show only bands with cutoff >= 64 Hz (skip the 32 Hz band at index 0)
        let band_cols: Vec<gpui::AnyElement> = modal
            .eq_bands
            .iter()
            .enumerate()
            .filter(|(_, band)| band.cutoff >= 64)
            .map(|(i, band)| {
                let gain_frac = gain_to_fraction(band.gain);
                let gain_db = band.gain as f32 / 10.0;
                let gain_str = if band.gain >= 0 {
                    format!("+{:.1}", gain_db)
                } else {
                    format!("{:.1}", gain_db)
                };
                let freq_hz = band.cutoff;
                let freq_label = if freq_hz >= 1000 {
                    format!("{}k", freq_hz / 1000)
                } else {
                    format!("{}", freq_hz)
                };
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .w(px(54.0))
                    .child(
                        div()
                            .text_xs()
                            .font_family("JetBrains Mono")
                            .text_color(text_secondary)
                            .child(gain_str),
                    )
                    .child(
                        EqSlider::new(
                            ElementId::NamedInteger("eq_band".into(), i as u64),
                            gain_frac,
                            rgba(0xFFFFFF12),
                            rgba(0x6F00FFCC),
                            rgba(0xFF446680),
                        )
                        .on_change(move |frac, _, cx| {
                            let gain = fraction_to_gain(frac);
                            if let Some(b) =
                                cx.global_mut::<SettingsModal>().eq_bands.get_mut(i)
                            {
                                b.gain = gain;
                            }
                            let rt = cx.global::<crate::state::TokioHandle>().0.clone();
                            let bands = cx.global::<SettingsModal>().eq_bands.clone();
                            rt.spawn(crate::client::save_settings_all(save_eq_bands(&bands)));
                        }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_secondary)
                            .child(freq_label),
                    )
                    .into_any_element()
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .gap_5()
            .py_2()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(text_primary)
                                    .child("Equalizer"),
                            )
                            .child(toggle_pill("eq_toggle", eq_enabled, accent, eq_toggle_l)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(text_secondary)
                                    .child("Precut"),
                            )
                            .child(
                                div().w(px(120.0)).child(
                                    SeekBar::new(
                                        "eq_precut_bar",
                                        precut_frac,
                                        rgba(0xFFFFFF12),
                                        accent,
                                        px(4.0),
                                    )
                                    .on_seek(move |frac, _, cx| {
                                        let new_v = (frac * 240.0).round() as u32;
                                        cx.global_mut::<SettingsModal>().eq_precut = new_v;
                                        let rt = cx.global::<crate::state::TokioHandle>().0.clone();
                                        rt.spawn(crate::client::save_settings_all(save_single(|r| r.eq_precut = Some(new_v))));
                                    }),
                                ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_family("JetBrains Mono")
                                    .text_color(text_secondary)
                                    .w(px(60.0))
                                    .child(format!("-{:.1} dB", precut_db)),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .justify_between()
                            .h(px(190.0))
                            .w(px(30.0))
                            .mr_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_family("JetBrains Mono")
                                    .text_color(text_secondary)
                                    .child("+24"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_family("JetBrains Mono")
                                    .text_color(rgba(0xFFFFFF40))
                                    .child(" 0"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_family("JetBrains Mono")
                                    .text_color(text_secondary)
                                    .child("-24"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_1()
                            .children(band_cols),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgba(0xFFFFFF40))
                    .child("Click a band to set gain."),
            )
            .into_any_element()
    }

    fn render_playback_tab(
        &self,
        theme: &Theme,
        modal: &SettingsModal,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let text_secondary = theme.library_header_text;
        let text_primary = theme.library_text;
        let accent = theme.switcher_active;
        let track_color = rgba(0xFFFFFF12);
        let border = theme.border;

        let crossfade = modal.crossfade;
        let fo_mix = modal.crossfade_fade_out_mixmode;
        let rg_type = modal.replaygain_type;
        let rg_preamp = modal.replaygain_preamp;
        let rg_noclip = modal.replaygain_noclip;
        let shuffle = modal.shuffle;
        let fi_delay = modal.crossfade_fade_in_delay;
        let fi_dur = modal.crossfade_fade_in_duration;
        let fo_delay = modal.crossfade_fade_out_delay;
        let fo_dur = modal.crossfade_fade_out_duration;

        let shuffle_l = cx.listener(|_, _, _, cx| {
            let v = cx.global::<SettingsModal>().shuffle;
            cx.global_mut::<SettingsModal>().shuffle = !v;
            let rt = cx.global::<crate::state::TokioHandle>().0.clone();
            rt.spawn(crate::client::save_settings_all(
                save_single(|r| r.playlist_shuffle = Some(!v)),
            ));
            cx.notify();
        });
        let crossfade_l = cx.listener(|_, _, _, cx| {
            let v = cx.global::<SettingsModal>().crossfade;
            let new_v = (v + 1) % 6;
            cx.global_mut::<SettingsModal>().crossfade = new_v;
            let rt = cx.global::<crate::state::TokioHandle>().0.clone();
            rt.spawn(crate::client::save_settings_all(
                save_single(|r| r.crossfade = Some(new_v)),
            ));
            cx.notify();
        });
        let fo_mix_l = cx.listener(|_, _, _, cx| {
            let v = cx.global::<SettingsModal>().crossfade_fade_out_mixmode;
            let new_v = (v + 1) % 2;
            cx.global_mut::<SettingsModal>().crossfade_fade_out_mixmode = new_v;
            let rt = cx.global::<crate::state::TokioHandle>().0.clone();
            rt.spawn(crate::client::save_settings_all(
                save_single(|r| r.fade_out_mixmode = Some(new_v)),
            ));
            cx.notify();
        });
        let rg_type_l = cx.listener(|_, _, _, cx| {
            let v = cx.global::<SettingsModal>().replaygain_type;
            let new_v = (v + 1) % 4;
            cx.global_mut::<SettingsModal>().replaygain_type = new_v;
            let rt = cx.global::<crate::state::TokioHandle>().0.clone();
            use crate::api::v1alpha1::ReplaygainSettings;
            let modal = cx.global::<SettingsModal>();
            let rg = ReplaygainSettings { r#type: new_v, preamp: modal.replaygain_preamp, noclip: modal.replaygain_noclip };
            rt.spawn(crate::client::save_settings_all(
                save_single(|r| r.replaygain_settings = Some(rg)),
            ));
            cx.notify();
        });
        let rg_noclip_l = cx.listener(|_, _, _, cx| {
            let v = cx.global::<SettingsModal>().replaygain_noclip;
            cx.global_mut::<SettingsModal>().replaygain_noclip = !v;
            let rt = cx.global::<crate::state::TokioHandle>().0.clone();
            use crate::api::v1alpha1::ReplaygainSettings;
            let modal = cx.global::<SettingsModal>();
            let rg = ReplaygainSettings { r#type: modal.replaygain_type, preamp: modal.replaygain_preamp, noclip: !v };
            rt.spawn(crate::client::save_settings_all(
                save_single(|r| r.replaygain_settings = Some(rg)),
            ));
            cx.notify();
        });

        div()
            .flex()
            .flex_col()
            .gap_4()
            .py_2()
            .child(section_header("Playback", text_secondary))
            .child(toggle_row_el(
                "Shuffle",
                "Randomly order tracks in the queue",
                shuffle,
                text_primary,
                text_secondary,
                accent,
                shuffle_l,
            ))
            .child(section_header("Crossfade", text_secondary))
            .child(cycle_row_el(
                "crossfade_mode",
                "Mode",
                crossfade_label(crossfade),
                text_primary,
                text_secondary,
                track_color,
                border,
                crossfade_l,
            ))
            .child(slider_row_el(
                "fi_delay",
                "Fade-in Delay",
                format!("{} s", fi_delay),
                fi_delay as f32 / 7.0,
                text_primary,
                text_secondary,
                accent,
                |frac, _, cx| {
                    let new_v = (frac * 7.0).round() as i32;
                    cx.global_mut::<SettingsModal>().crossfade_fade_in_delay = new_v;
                    let rt = cx.global::<crate::state::TokioHandle>().0.clone();
                    rt.spawn(crate::client::save_settings_all(save_single(|r| r.fade_in_delay = Some(new_v))));
                },
            ))
            .child(slider_row_el(
                "fi_dur",
                "Fade-in Duration",
                format!("{} s", fi_dur),
                fi_dur as f32 / 15.0,
                text_primary,
                text_secondary,
                accent,
                |frac, _, cx| {
                    let new_v = (frac * 15.0).round() as i32;
                    cx.global_mut::<SettingsModal>().crossfade_fade_in_duration = new_v;
                    let rt = cx.global::<crate::state::TokioHandle>().0.clone();
                    rt.spawn(crate::client::save_settings_all(save_single(|r| r.fade_in_duration = Some(new_v))));
                },
            ))
            .child(slider_row_el(
                "fo_delay",
                "Fade-out Delay",
                format!("{} s", fo_delay),
                fo_delay as f32 / 7.0,
                text_primary,
                text_secondary,
                accent,
                |frac, _, cx| {
                    let new_v = (frac * 7.0).round() as i32;
                    cx.global_mut::<SettingsModal>().crossfade_fade_out_delay = new_v;
                    let rt = cx.global::<crate::state::TokioHandle>().0.clone();
                    rt.spawn(crate::client::save_settings_all(save_single(|r| r.fade_out_delay = Some(new_v))));
                },
            ))
            .child(slider_row_el(
                "fo_dur",
                "Fade-out Duration",
                format!("{} s", fo_dur),
                fo_dur as f32 / 15.0,
                text_primary,
                text_secondary,
                accent,
                |frac, _, cx| {
                    let new_v = (frac * 15.0).round() as i32;
                    cx.global_mut::<SettingsModal>().crossfade_fade_out_duration = new_v;
                    let rt = cx.global::<crate::state::TokioHandle>().0.clone();
                    rt.spawn(crate::client::save_settings_all(save_single(|r| r.fade_out_duration = Some(new_v))));
                },
            ))
            .child(cycle_row_el(
                "fo_mix",
                "Fade-out Mode",
                mixmode_label(fo_mix),
                text_primary,
                text_secondary,
                track_color,
                border,
                fo_mix_l,
            ))
            .child(section_header("ReplayGain", text_secondary))
            .child(cycle_row_el(
                "rg_type",
                "Mode",
                replaygain_label(rg_type),
                text_primary,
                text_secondary,
                track_color,
                border,
                rg_type_l,
            ))
            .child(slider_row_el(
                "rg_preamp",
                "Pre-amp",
                format!("{:.1} dB", rg_preamp as f32 / 10.0),
                (rg_preamp + 120) as f32 / 240.0,
                text_primary,
                text_secondary,
                accent,
                |frac, _, cx| {
                    let new_v = (frac * 240.0 - 120.0).round() as i32;
                    cx.global_mut::<SettingsModal>().replaygain_preamp = new_v;
                    let rt = cx.global::<crate::state::TokioHandle>().0.clone();
                    let m = cx.global::<SettingsModal>();
                    let rg = crate::api::v1alpha1::ReplaygainSettings {
                        noclip: m.replaygain_noclip,
                        r#type: m.replaygain_type,
                        preamp: new_v,
                    };
                    rt.spawn(crate::client::save_settings_all(save_single(|r| r.replaygain_settings = Some(rg))));
                },
            ))
            .child(toggle_row_el(
                "Prevent Clipping",
                "Reduce gain to avoid clipping",
                rg_noclip,
                text_primary,
                text_secondary,
                accent,
                rg_noclip_l,
            ))
            .into_any_element()
    }

    fn render_sound_tab(
        &self,
        theme: &Theme,
        modal: &SettingsModal,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let text_secondary = theme.library_header_text;
        let text_primary = theme.library_text;
        let accent = theme.switcher_active;
        let track_color = rgba(0xFFFFFF12);
        let border = theme.border;

        let balance = modal.balance;
        let bass = modal.bass;
        let treble = modal.treble;
        let stereo_width = modal.stereo_width;
        let channel_config = modal.channel_config;
        let surround = modal.surround_enabled;
        let dithering = modal.dithering_enabled;

        let channel_l = cx.listener(|_, _, _, cx| {
            let v = cx.global::<SettingsModal>().channel_config;
            let new_v = (v + 1) % 6;
            cx.global_mut::<SettingsModal>().channel_config = new_v;
            let rt = cx.global::<crate::state::TokioHandle>().0.clone();
            rt.spawn(crate::client::save_settings_all(save_single(|r| r.channel_config = Some(new_v))));
            cx.notify();
        });
        let dithering_l = cx.listener(|_, _, _, cx| {
            let v = cx.global::<SettingsModal>().dithering_enabled;
            cx.global_mut::<SettingsModal>().dithering_enabled = !v;
            // dithering_enabled has no dedicated field in SaveSettingsRequest; skip remote save
            cx.notify();
        });

        div()
            .flex()
            .flex_col()
            .gap_4()
            .py_2()
            .child(section_header("Tone & Stereo", text_secondary))
            .child(slider_row_el(
                "balance",
                "Balance",
                format!("{:+}", balance),
                (balance + 100) as f32 / 200.0,
                text_primary,
                text_secondary,
                accent,
                |frac, _, cx| {
                    let new_v = (frac * 200.0 - 100.0).round() as i32;
                    cx.global_mut::<SettingsModal>().balance = new_v;
                    let rt = cx.global::<crate::state::TokioHandle>().0.clone();
                    rt.spawn(crate::client::save_settings_all(save_single(|r| r.balance = Some(new_v))));
                },
            ))
            .child(slider_row_el(
                "bass",
                "Bass",
                format!("{:+} dB", bass),
                (bass + 24) as f32 / 48.0,
                text_primary,
                text_secondary,
                accent,
                |frac, _, cx| {
                    let new_v = (frac * 48.0 - 24.0).round() as i32;
                    cx.global_mut::<SettingsModal>().bass = new_v;
                    let rt = cx.global::<crate::state::TokioHandle>().0.clone();
                    rt.spawn(crate::client::save_settings_all(save_single(|r| r.bass = Some(new_v))));
                },
            ))
            .child(slider_row_el(
                "treble",
                "Treble",
                format!("{:+} dB", treble),
                (treble + 24) as f32 / 48.0,
                text_primary,
                text_secondary,
                accent,
                |frac, _, cx| {
                    let new_v = (frac * 48.0 - 24.0).round() as i32;
                    cx.global_mut::<SettingsModal>().treble = new_v;
                    let rt = cx.global::<crate::state::TokioHandle>().0.clone();
                    rt.spawn(crate::client::save_settings_all(save_single(|r| r.treble = Some(new_v))));
                },
            ))
            .child(slider_row_el(
                "stereo_width",
                "Stereo Width",
                format!("{}%", stereo_width),
                stereo_width as f32 / 255.0,
                text_primary,
                text_secondary,
                accent,
                |frac, _, cx| {
                    let new_v = (frac * 255.0).round() as i32;
                    cx.global_mut::<SettingsModal>().stereo_width = new_v;
                    let rt = cx.global::<crate::state::TokioHandle>().0.clone();
                    rt.spawn(crate::client::save_settings_all(save_single(|r| r.stereo_width = Some(new_v))));
                },
            ))
            .child(cycle_row_el(
                "channel_config",
                "Channel Config",
                channel_label(channel_config),
                text_primary,
                text_secondary,
                track_color,
                border,
                channel_l,
            ))
            .child(section_header("DSP", text_secondary))
            .child(slider_row_el(
                "surround",
                "Surround",
                if surround > 0 { format!("Level {}", surround) } else { "Off".into() },
                surround as f32 / 10.0,
                text_primary,
                text_secondary,
                accent,
                |frac, _, cx| {
                    let new_v = (frac * 10.0).round() as i32;
                    cx.global_mut::<SettingsModal>().surround_enabled = new_v;
                    let rt = cx.global::<crate::state::TokioHandle>().0.clone();
                    rt.spawn(crate::client::save_settings_all(save_single(|r| r.surround_enabled = Some(new_v))));
                },
            ))
            .child(toggle_row_el(
                "Dithering",
                "Adds shaped noise to reduce quantization artifacts",
                dithering,
                text_primary,
                text_secondary,
                accent,
                dithering_l,
            ))
            .into_any_element()
    }
}

// ── UI primitives ─────────────────────────────────────────────────────────────

fn section_header(label: &'static str, text_secondary: gpui::Rgba) -> impl IntoElement {
    div()
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(text_secondary)
        .pt_2()
        .child(label)
}

fn setting_field(
    label: &str,
    hint: &str,
    text_primary: gpui::Rgba,
    text_secondary: gpui::Rgba,
    input: Entity<TextInput>,
) -> gpui::AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(text_primary)
                .child(label.to_string()),
        )
        .child(
            div()
                .text_xs()
                .text_color(text_secondary)
                .mb_1()
                .child(hint.to_string()),
        )
        .child(input)
        .into_any_element()
}

/// A pill-shaped on/off toggle. Accepts a pre-created listener.
fn toggle_pill(
    id: &'static str,
    on: bool,
    accent: gpui::Rgba,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let track_bg = if on { accent } else { rgba(0xFFFFFF20) };
    let thumb_x = if on { px(22.0) } else { px(2.0) };
    div()
        .id(ElementId::Name(id.into()))
        .w(px(44.0))
        .h(px(24.0))
        .rounded_full()
        .bg(track_bg)
        .relative()
        .cursor_pointer()
        .on_click(on_click)
        .child(
            div()
                .absolute()
                .top(px(2.0))
                .left(thumb_x)
                .w(px(20.0))
                .h(px(20.0))
                .rounded_full()
                .bg(gpui::white()),
        )
}

fn toggle_row_el(
    label: &str,
    hint: &str,
    on: bool,
    text_primary: gpui::Rgba,
    text_secondary: gpui::Rgba,
    accent: gpui::Rgba,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> gpui::AnyElement {
    let id = ElementId::Name(format!("tr_{}", label).into());
    let track_bg = if on { accent } else { rgba(0xFFFFFF20) };
    let thumb_x = if on { px(22.0) } else { px(2.0) };
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .py_1()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_px()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(text_primary)
                        .child(label.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(text_secondary)
                        .child(hint.to_string()),
                ),
        )
        .child(
            div()
                .id(id)
                .w(px(44.0))
                .h(px(24.0))
                .rounded_full()
                .bg(track_bg)
                .relative()
                .cursor_pointer()
                .on_click(on_click)
                .child(
                    div()
                        .absolute()
                        .top(px(2.0))
                        .left(thumb_x)
                        .w(px(20.0))
                        .h(px(20.0))
                        .rounded_full()
                        .bg(gpui::white()),
                ),
        )
        .into_any_element()
}

fn cycle_row_el(
    id: &'static str,
    label: &str,
    value: &str,
    text_primary: gpui::Rgba,
    text_secondary: gpui::Rgba,
    track_color: gpui::Rgba,
    border: gpui::Rgba,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> gpui::AnyElement {
    let value_str = value.to_string();
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .py_1()
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(text_primary)
                .child(label.to_string()),
        )
        .child(
            div()
                .id(ElementId::Name(id.into()))
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_3()
                .py_1()
                .rounded_lg()
                .bg(track_color)
                .border_1()
                .border_color(border)
                .cursor_pointer()
                .min_w(px(200.0))
                .hover(|d| d.bg(rgba(0xFFFFFF1F)))
                .on_click(on_click)
                .child(div().flex_1().text_sm().text_color(text_secondary).child(value_str))
                .child(div().text_xs().text_color(text_secondary).child("▾")),
        )
        .into_any_element()
}

fn slider_row_el(
    id: &'static str,
    label: &str,
    value_str: String,
    fraction: f32,
    text_primary: gpui::Rgba,
    text_secondary: gpui::Rgba,
    fill_color: gpui::Rgba,
    on_seek: impl Fn(f32, &mut gpui::Window, &mut gpui::App) + 'static,
) -> gpui::AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_4()
        .py_1()
        .child(
            div()
                .flex_shrink_0()
                .w(px(150.0))
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(text_primary)
                .child(label.to_string()),
        )
        .child(
            div().flex_1().child(
                SeekBar::new(id, fraction.clamp(0.0, 1.0), rgba(0xFFFFFF12), fill_color, px(4.0))
                    .on_seek(on_seek),
            ),
        )
        .child(
            div()
                .flex_shrink_0()
                .w(px(72.0))
                .text_right()
                .text_xs()
                .font_family("JetBrains Mono")
                .text_color(text_secondary)
                .child(value_str),
        )
        .into_any_element()
}
