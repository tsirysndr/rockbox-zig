use crate::api::rockbox::v1alpha1::settings_service_client::SettingsServiceClient;
use crate::api::rockbox::v1alpha1::{
    GetAlbumResponse, GetGlobalSettingsRequest, GetGlobalSettingsResponse, ReplaygainSettings,
    SaveSettingsRequest, SaveSettingsResponse,
};
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use gtk::glib::property::PropertyGet;
use gtk::{glib, CompositeTemplate};
use std::cell::RefCell;
use std::{env, thread};

macro_rules! connect_equalizer_band_tooltips {
    ($self:expr, $($band:ident), * $(,)?) => {
        $(
            $self.imp().$band.set_has_tooltip(true);
            $self.imp().$band.connect_value_changed(|s| {
                let value = s.value();
                s.set_tooltip_text(Some(&format!("{:.1} dB", value)));

            });
            $self.imp().$band.connect_query_tooltip(|s, _x, _y, _keyboard_mode, tooltip| {
                let value = s.value();
                tooltip.set_text(Some(&format!("{:.1} dB", value)));
                true
            });
        )+
    };
}

macro_rules! set_equalizer_bands {
    ($self:expr, $settings:expr, $($index:expr),+ $(,)?) => {
        $(
            match $index {
                0 => $self.imp().equalizer_band_1.set_value($settings.eq_band_settings[$index].cutoff as f64 * 0.1),
                1 => $self.imp().equalizer_band_2.set_value($settings.eq_band_settings[$index].cutoff as f64 * 0.1),
                2 => $self.imp().equalizer_band_3.set_value($settings.eq_band_settings[$index].cutoff as f64 * 0.1),
                3 => $self.imp().equalizer_band_4.set_value($settings.eq_band_settings[$index].cutoff as f64 * 0.1),
                4 => $self.imp().equalizer_band_5.set_value($settings.eq_band_settings[$index].cutoff as f64 * 0.1),
                5 => $self.imp().equalizer_band_6.set_value($settings.eq_band_settings[$index].cutoff as f64 * 0.1),
                6 => $self.imp().equalizer_band_7.set_value($settings.eq_band_settings[$index].cutoff as f64 * 0.1),
                7 => $self.imp().equalizer_band_8.set_value($settings.eq_band_settings[$index].cutoff as f64 * 0.1),
                8 => $self.imp().equalizer_band_9.set_value($settings.eq_band_settings[$index].cutoff as f64 * 0.1),
                9 => $self.imp().equalizer_band_10.set_value($settings.eq_band_settings[$index].cutoff as f64 * 0.1),
                _ => panic!("Invalid equalizer band index")
            }
        )+
    };
}

macro_rules! connect_equalizer_band_value_changed {
    ($self:expr, $band:ident, $index:expr) => {
        let self_weak = $self.downgrade();
        $self.$band.connect_value_changed(move |scale| {
            let self_ = match self_weak.upgrade() {
                Some(self_) => self_,
                None => return,
            };
            let value = scale.value();
            let obj = self_.obj();
            {
                let mut settings = obj.imp().settings.borrow_mut();
                if let Some(settings) = settings.as_mut() {
                    settings.eq_band_settings[$index].cutoff = (value * 10.0) as i32;
                }
            }
            obj.save_settings();
        });
    };
}

mod imp {

    use glib::subclass;

    use crate::api::rockbox::v1alpha1::ReplaygainSettings;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/mg/tsirysndr/Rockbox/gtk/preferences.ui")]
    pub struct RbPreferencesDialog {
        #[template_child]
        pub library_location_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub directory_picker_button: TemplateChild<gtk::Button>,
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

        pub settings: RefCell<Option<SaveSettingsRequest>>,
        pub replaygain_settings: RefCell<Option<ReplaygainSettings>>,
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
            self.settings.replace(None);

            self.handle_music_dir();
            self.handle_bass();
            self.handle_treble();
            self.handle_balance();
            self.handle_enable_equalizer();
            self.handle_equalizer_bands();
            self.handle_repeat();
            self.handle_shuffle();
            self.handle_fade_on_stop();
            self.handle_crossfade();
            self.handle_fade_in_delay();
            self.handle_fade_in_duration();
            self.handle_fade_out_delay();
            self.handle_fade_out_duration();
            self.handle_fade_out_mode();
            self.handle_replaygain();

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::spawn_future_local(async move {
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

    impl RbPreferencesDialog {
        fn handle_music_dir(&self) {
            let self_weak = self.downgrade();
            self.directory_picker_button.connect_clicked(move |_| {
                let dialog = gtk::FileChooserDialog::builder()
                    .title("Select Music Library Location")
                    .action(gtk::FileChooserAction::SelectFolder)
                    .build();

                dialog.add_buttons(&[
                    ("Cancel", gtk::ResponseType::Cancel),
                    ("Select", gtk::ResponseType::Accept),
                ]);

                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return,
                };

                dialog.connect_response(move |dialog, response| {
                    if response != gtk::ResponseType::Accept {
                        dialog.close();
                        return;
                    }

                    if let Some(folder) = dialog.file() {
                        let obj = self_.obj();
                        let path = folder.path().unwrap();
                        let path = path.to_str().unwrap();
                        obj.imp().library_location_label.set_label(path);
                        {
                            let mut settings = obj.imp().settings.borrow_mut();
                            if let Some(settings) = settings.as_mut() {
                                settings.music_dir = Some(path.to_string());
                            }
                        }

                        obj.save_settings();
                    }
                    dialog.close();
                });

                dialog.show();
            });
        }

        fn handle_bass(&self) {
            let self_weak = self.downgrade();
            self.bass
                .connect_notify_local(Some("value"), move |spin_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = spin_row.value();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.bass = Some(value as i32);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_treble(&self) {
            let self_weak = self.downgrade();
            self.treble
                .connect_notify_local(Some("value"), move |spin_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = spin_row.value();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.treble = Some(value as i32);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_balance(&self) {
            let self_weak = self.downgrade();
            self.balance
                .connect_notify_local(Some("value"), move |spin_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = spin_row.value();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.balance = Some(value as i32);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_enable_equalizer(&self) {
            let self_weak = self.downgrade();
            self.enable_equalizer
                .connect_notify_local(Some("state"), move |switch_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = switch_row.is_active();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.eq_enabled = Some(value);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_equalizer_bands(&self) {
            connect_equalizer_band_value_changed!(self, equalizer_band_1, 0);
            connect_equalizer_band_value_changed!(self, equalizer_band_2, 1);
            connect_equalizer_band_value_changed!(self, equalizer_band_3, 2);
            connect_equalizer_band_value_changed!(self, equalizer_band_4, 3);
            connect_equalizer_band_value_changed!(self, equalizer_band_5, 4);
            connect_equalizer_band_value_changed!(self, equalizer_band_6, 5);
            connect_equalizer_band_value_changed!(self, equalizer_band_7, 6);
            connect_equalizer_band_value_changed!(self, equalizer_band_8, 7);
            connect_equalizer_band_value_changed!(self, equalizer_band_9, 8);
            connect_equalizer_band_value_changed!(self, equalizer_band_10, 9);
        }

        fn handle_repeat(&self) {
            let self_weak = self.downgrade();
            self.repeat
                .connect_notify_local(Some("selected"), move |combo_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = combo_row.selected();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.repeat_mode = Some(value as i32);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_shuffle(&self) {
            let self_weak = self.downgrade();
            self.shuffle
                .connect_state_flags_changed(move |switch_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = switch_row.is_active();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.playlist_shuffle = Some(value);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_fade_on_stop(&self) {
            let self_weak = self.downgrade();
            self.fade_on_stop
                .connect_state_flags_changed(move |switch_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = switch_row.is_active();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.fade_on_stop = Some(value);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_crossfade(&self) {
            let self_weak = self.downgrade();
            self.crossfade
                .connect_notify_local(Some("selected"), move |combo_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = combo_row.selected();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.crossfade = Some(value as i32);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_fade_in_delay(&self) {
            let self_weak = self.downgrade();
            self.fade_in_delay
                .connect_notify_local(Some("value"), move |spin_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = spin_row.value();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.fade_in_delay = Some(value as i32);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_fade_in_duration(&self) {
            let self_weak = self.downgrade();
            self.fade_in_duration
                .connect_notify_local(Some("value"), move |spin_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = spin_row.value();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.fade_in_duration = Some(value as i32);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_fade_out_delay(&self) {
            let self_weak = self.downgrade();
            self.fade_out_delay
                .connect_notify_local(Some("value"), move |spin_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = spin_row.value();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.fade_out_delay = Some(value as i32);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_fade_out_duration(&self) {
            let self_weak = self.downgrade();
            self.fade_out_duration
                .connect_notify_local(Some("value"), move |spin_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = spin_row.value();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.fade_out_duration = Some(value as i32);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_fade_out_mode(&self) {
            let self_weak = self.downgrade();
            self.fade_out_mode
                .connect_notify_local(Some("selected"), move |combo_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = combo_row.selected();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            settings.fade_out_mixmode = Some(value as i32);
                        }
                    }
                    obj.save_settings();
                });
        }

        fn handle_replaygain(&self) {
            let self_weak = self.downgrade();
            self.replaygain
                .connect_notify_local(Some("selected"), move |combo_row, _| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };
                    let value = combo_row.selected();
                    let obj = self_.obj();
                    {
                        let mut settings = obj.imp().settings.borrow_mut();
                        if let Some(settings) = settings.as_mut() {
                            if let Some(ref mut replaygain_settings) = settings.replaygain_settings
                            {
                                replaygain_settings.r#type = value as i32;
                            } else {
                                let replaygain_settings =
                                    obj.imp().replaygain_settings.borrow_mut();
                                let replaygain_settings = replaygain_settings.as_ref().unwrap();
                                let mut replaygain_settings = replaygain_settings.clone();
                                replaygain_settings.r#type = value as i32;
                                settings.replaygain_settings = Some(replaygain_settings);
                            }
                        }
                    }
                    obj.save_settings();
                });
        }
    }
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
                self.imp().settings.replace(Some(SaveSettingsRequest {
                    eq_band_settings: settings.eq_band_settings.clone(),
                    ..Default::default()
                }));
                self.imp()
                    .replaygain_settings
                    .replace(settings.replaygain_settings);
                self.imp()
                    .library_location_label
                    .set_label(&settings.music_dir);
                self.imp().bass.set_value(settings.bass as f64);
                self.imp().treble.set_value(settings.treble as f64);
                self.imp().balance.set_value(settings.treble as f64);
                self.imp().enable_equalizer.set_active(settings.eq_enabled);

                set_equalizer_bands!(self, settings, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9);

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
        connect_equalizer_band_tooltips!(
            self,
            equalizer_band_1,
            equalizer_band_2,
            equalizer_band_3,
            equalizer_band_4,
            equalizer_band_5,
            equalizer_band_6,
            equalizer_band_7,
            equalizer_band_8,
            equalizer_band_9,
            equalizer_band_10,
        );
    }

    fn save_settings(&self) {
        let settings = self.imp().settings.borrow();
        let settings_ref = settings.as_ref();
        if let Some(settings) = settings.clone() {
            thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let response = rt.block_on(async {
                    let url = build_url();
                    let mut client = SettingsServiceClient::connect(url).await?;
                    let response = client.save_settings(settings).await?.into_inner();
                    Ok::<SaveSettingsResponse, Error>(response)
                });
                if let Err(e) = response {
                    eprintln!("Error saving settings: {}", e);
                }
            });
        }
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    format!("tcp://{}:{}", host, port)
}
