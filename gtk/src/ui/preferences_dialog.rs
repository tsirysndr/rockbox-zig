use crate::api::rockbox::v1alpha1::settings_service_client::SettingsServiceClient;
use crate::api::rockbox::v1alpha1::{
    GetAlbumResponse, GetGlobalSettingsRequest, GetGlobalSettingsResponse,
};
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use gtk::{glib, CompositeTemplate};
use std::{env, thread};

mod imp {
    use glib::subclass;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "gtk/preferences.ui")]
    pub struct RbPreferencesDialog {
        #[template_child]
        pub library_location_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub file_chooser_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub bass: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub treble: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub balance: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub enable_equalizer: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub equalizer_band_1: TemplateChild<gtk::Scale>,
        #[template_child]
        pub equalizer_band_2: TemplateChild<gtk::Scale>,
        #[template_child]
        pub equalizer_band_3: TemplateChild<gtk::Scale>,
        #[template_child]
        pub equalizer_band_4: TemplateChild<gtk::Scale>,
        #[template_child]
        pub equalizer_band_5: TemplateChild<gtk::Scale>,
        #[template_child]
        pub equalizer_band_6: TemplateChild<gtk::Scale>,
        #[template_child]
        pub equalizer_band_7: TemplateChild<gtk::Scale>,
        #[template_child]
        pub equalizer_band_8: TemplateChild<gtk::Scale>,
        #[template_child]
        pub equalizer_band_9: TemplateChild<gtk::Scale>,
        #[template_child]
        pub equalizer_band_10: TemplateChild<gtk::Scale>,
        #[template_child]
        pub repeat: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub shuffle: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub fade_on_stop: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub crossfade: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub fade_in_delay: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub fade_in_duration: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub fade_out_delay: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub fade_out_duration: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub fade_out_mode: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub replaygain: TemplateChild<adw::ComboRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RbPreferencesDialog {
        const NAME: &'static str = "RbPreferencesDialog";
        type ParentType = adw::PreferencesDialog;
        type Type = super::RbPreferencesDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RbPreferencesDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::MainContext::default().spawn_local(async move {
                    let obj = self_.obj();
                    obj.load_settings();
                });

                glib::ControlFlow::Break
            });
        }
    }

    impl WidgetImpl for RbPreferencesDialog {}

    impl AdwDialogImpl for RbPreferencesDialog {}

    impl PreferencesDialogImpl for RbPreferencesDialog {}

    impl RbPreferencesDialog {}
}

glib::wrapper! {
    pub struct RbPreferencesDialog(ObjectSubclass<imp::RbPreferencesDialog>)
    @extends gtk::Widget, adw::Dialog, adw::PreferencesDialog;
}

impl Default for RbPreferencesDialog {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl RbPreferencesDialog {
    pub fn load_settings(&self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response = rt.block_on(async {
            let url = build_url();
            let mut client = SettingsServiceClient::connect(url).await?;
            let response = client
                .get_global_settings(GetGlobalSettingsRequest {})
                .await?
                .into_inner();
            Ok::<GetGlobalSettingsResponse, Error>(response)
        });

        match response {
            Ok(settings) => {
                self.imp()
                    .library_location_label
                    .set_label(&settings.music_dir);
                self.imp().bass.set_value(settings.bass as f64);
                self.imp().treble.set_value(settings.treble as f64);
                self.imp().balance.set_value(settings.treble as f64);
                self.imp().enable_equalizer.set_active(settings.eq_enabled);
                self.imp()
                    .equalizer_band_1
                    .set_value(settings.eq_band_settings[0].cutoff as f64 * 0.1);
                self.imp()
                    .equalizer_band_2
                    .set_value(settings.eq_band_settings[1].cutoff as f64 * 0.1);
                self.imp()
                    .equalizer_band_3
                    .set_value(settings.eq_band_settings[2].cutoff as f64 * 0.1);
                self.imp()
                    .equalizer_band_4
                    .set_value(settings.eq_band_settings[3].cutoff as f64 * 0.1);
                self.imp()
                    .equalizer_band_5
                    .set_value(settings.eq_band_settings[4].cutoff as f64 * 0.1);
                self.imp()
                    .equalizer_band_6
                    .set_value(settings.eq_band_settings[5].cutoff as f64 * 0.1);
                self.imp()
                    .equalizer_band_7
                    .set_value(settings.eq_band_settings[6].cutoff as f64 * 0.1);
                self.imp()
                    .equalizer_band_8
                    .set_value(settings.eq_band_settings[7].cutoff as f64 * 0.1);
                self.imp()
                    .equalizer_band_9
                    .set_value(settings.eq_band_settings[8].cutoff as f64 * 0.1);
                self.imp()
                    .equalizer_band_10
                    .set_value(settings.eq_band_settings[9].cutoff as f64 * 0.1);
                self.imp().repeat.set_selected(settings.repeat_mode as u32);
                self.imp().shuffle.set_active(settings.playlist_shuffle);
                self.imp().fade_on_stop.set_active(settings.fade_on_stop);
                self.imp().crossfade.set_selected(settings.crossfade as u32);
                self.imp()
                    .fade_in_delay
                    .set_value(settings.crossfade_fade_in_delay as f64);
                self.imp()
                    .fade_in_duration
                    .set_value(settings.crossfade_fade_in_duration as f64);
                self.imp()
                    .fade_out_delay
                    .set_value(settings.crossfade_fade_out_delay as f64);
                self.imp()
                    .fade_out_duration
                    .set_value(settings.crossfade_fade_out_duration as f64);
                self.imp()
                    .fade_out_mode
                    .set_selected(settings.crossfade_fade_out_mixmode as u32);
                self.imp().replaygain.set_selected(
                    settings
                        .replaygain_settings
                        .map(|r| r.r#type as u32)
                        .unwrap_or(0),
                );
                self.set_equalizer_tooltip();
            }
            Err(e) => {
                eprintln!("Error loading settings: {}", e);
            }
        }
    }

    fn set_equalizer_tooltip(&self) {
        self.imp().equalizer_band_1.set_has_tooltip(true);
        self.imp().equalizer_band_2.set_has_tooltip(true);
        self.imp().equalizer_band_3.set_has_tooltip(true);
        self.imp().equalizer_band_4.set_has_tooltip(true);
        self.imp().equalizer_band_5.set_has_tooltip(true);
        self.imp().equalizer_band_6.set_has_tooltip(true);
        self.imp().equalizer_band_7.set_has_tooltip(true);
        self.imp().equalizer_band_8.set_has_tooltip(true);
        self.imp().equalizer_band_9.set_has_tooltip(true);
        self.imp().equalizer_band_10.set_has_tooltip(true);

        self.imp().equalizer_band_1.connect_value_changed(|s| {
            let value = s.value();
            s.set_tooltip_text(Some(&format!("{:.1} dB", value)));
        });

        self.imp().equalizer_band_2.connect_value_changed(|s| {
            let value = s.value();
            s.set_tooltip_text(Some(&format!("{:.1} dB", value)));
        });

        self.imp().equalizer_band_3.connect_value_changed(|s| {
            let value = s.value();
            s.set_tooltip_text(Some(&format!("{:.1} dB", value)));
        });

        self.imp().equalizer_band_4.connect_value_changed(|s| {
            let value = s.value();
            s.set_tooltip_text(Some(&format!("{:.1} dB", value)));
        });

        self.imp().equalizer_band_5.connect_value_changed(|s| {
            let value = s.value();
            s.set_tooltip_text(Some(&format!("{:.1} dB", value)));
        });

        self.imp().equalizer_band_6.connect_value_changed(|s| {
            let value = s.value();
            s.set_tooltip_text(Some(&format!("{:.1} dB", value)));
        });

        self.imp().equalizer_band_7.connect_value_changed(|s| {
            let value = s.value();
            s.set_tooltip_text(Some(&format!("{:.1} dB", value)));
        });

        self.imp().equalizer_band_8.connect_value_changed(|s| {
            let value = s.value();
            s.set_tooltip_text(Some(&format!("{:.1} dB", value)));
        });

        self.imp().equalizer_band_9.connect_value_changed(|s| {
            let value = s.value();
            s.set_tooltip_text(Some(&format!("{:.1} dB", value)));
        });

        self.imp().equalizer_band_10.connect_value_changed(|s| {
            let value = s.value();
            s.set_tooltip_text(Some(&format!("{:.1} dB", value)));
        });

        self.imp()
            .equalizer_band_1
            .connect_query_tooltip(|s, _x, _y, _keyboad_mode, tooltip| {
                let value = s.value();
                tooltip.set_text(Some(&format!("{:.1} dB", value)));
                true
            });

        self.imp()
            .equalizer_band_2
            .connect_query_tooltip(|s, _x, _y, _keyboad_mode, tooltip| {
                let value = s.value();
                tooltip.set_text(Some(&format!("{:.1} dB", value)));
                true
            });

        self.imp()
            .equalizer_band_3
            .connect_query_tooltip(|s, _x, _y, _keyboad_mode, tooltip| {
                let value = s.value();
                tooltip.set_text(Some(&format!("{:.1} dB", value)));
                true
            });

        self.imp()
            .equalizer_band_4
            .connect_query_tooltip(|s, _x, _y, _keyboad_mode, tooltip| {
                let value = s.value();
                tooltip.set_text(Some(&format!("{:.1} dB", value)));
                true
            });

        self.imp()
            .equalizer_band_5
            .connect_query_tooltip(|s, _x, _y, _keyboad_mode, tooltip| {
                let value = s.value();
                tooltip.set_text(Some(&format!("{:.1} dB", value)));
                true
            });

        self.imp()
            .equalizer_band_6
            .connect_query_tooltip(|s, _x, _y, _keyboad_mode, tooltip| {
                let value = s.value();
                tooltip.set_text(Some(&format!("{:.1} dB", value)));
                true
            });

        self.imp()
            .equalizer_band_7
            .connect_query_tooltip(|s, _x, _y, _keyboad_mode, tooltip| {
                let value = s.value();
                tooltip.set_text(Some(&format!("{:.1} dB", value)));
                true
            });

        self.imp()
            .equalizer_band_8
            .connect_query_tooltip(|s, _x, _y, _keyboad_mode, tooltip| {
                let value = s.value();
                tooltip.set_text(Some(&format!("{:.1} dB", value)));
                true
            });

        self.imp()
            .equalizer_band_9
            .connect_query_tooltip(|s, _x, _y, _keyboad_mode, tooltip| {
                let value = s.value();
                tooltip.set_text(Some(&format!("{:.1} dB", value)));
                true
            });

        self.imp()
            .equalizer_band_10
            .connect_query_tooltip(|s, _x, _y, _keyboad_mode, tooltip| {
                let value = s.value();
                tooltip.set_text(Some(&format!("{:.1} dB", value)));
                true
            });
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    format!("tcp://{}:{}", host, port)
}
