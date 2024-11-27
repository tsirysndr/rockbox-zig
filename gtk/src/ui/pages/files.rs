use crate::api::rockbox::v1alpha1::browse_service_client::BrowseServiceClient;
use crate::api::rockbox::v1alpha1::{TreeGetEntriesRequest, TreeGetEntriesResponse};
use crate::ui::file::File;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::{CompositeTemplate, ListBox};
use std::env;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "../gtk/files.ui")]
    pub struct Files {
        #[template_child]
        pub files: TemplateChild<ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Files {
        const NAME: &'static str = "Files";
        type ParentType = gtk::Box;
        type Type = super::Files;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Files {
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
                    obj.load_files(None);
                });
                glib::ControlFlow::Break
            });
        }
    }

    impl WidgetImpl for Files {}
    impl BoxImpl for Files {}
}

glib::wrapper! {
  pub struct Files(ObjectSubclass<imp::Files>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Files {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_files(&self, path: Option<String>) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response_ = rt.block_on(async {
            let url = build_url();
            let mut client = BrowseServiceClient::connect(url).await?;
            let response = client
                .tree_get_entries(TreeGetEntriesRequest { path })
                .await?
                .into_inner();
            Ok::<TreeGetEntriesResponse, Error>(response)
        });

        if let Ok(response) = response_ {
            let files = self.imp().files.clone();
            while let Some(row) = files.first_child() {
                files.remove(&row);
            }

            for entry in response.entries {
                let file = File::new();
                // pop up the filename
                let filename = entry.name.split("/").last().unwrap();
                file.imp().file_name.set_text(filename);
                self.imp().files.append(&file);
            }
        }
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or("localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or("6061".to_string());
    format!("tcp://{}:{}", host, port)
}
