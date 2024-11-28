use crate::api::rockbox::v1alpha1::browse_service_client::BrowseServiceClient;
use crate::api::rockbox::v1alpha1::{TreeGetEntriesRequest, TreeGetEntriesResponse};
use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::{CompositeTemplate, Image, Label, ListBox};
use adw::prelude::*;
use std::cell::{RefCell, Cell};
use anyhow::Error;
use std::env;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "./gtk/file.ui")]
    pub struct File {
        #[template_child]
        pub file_icon: TemplateChild<Image>,
        #[template_child]
        pub file_name: TemplateChild<Label>,
        #[template_child]
        pub row: TemplateChild<gtk::Box>,

        pub files: RefCell<Option<ListBox>>,
        pub go_back_button: RefCell<Option<gtk::Button>>,
        pub path: RefCell<String>,
        pub current_path: RefCell<Option<String>>,
        pub is_dir: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for File {
        const NAME: &'static str = "File";
        type ParentType = gtk::Box;
        type Type = super::File;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for File {
        fn constructed(&self) {
            self.parent_constructed();

            let self_weak = self.downgrade();
            let click = gtk::GestureClick::new();
            click.connect_released(move |_, _, _, _| {
                if let Some(self_) = self_weak.upgrade() {
                    let path = self_.path.borrow();
                    let path = path.clone();
                    let obj = self_.obj();

                    if !self_.is_dir.get() {
                        return;
                    }

                    let mut current_path = self_.current_path.borrow_mut();
                    *current_path = Some(path.clone());

                    obj.load_files(Some(path));
                    let go_back_button = self_.go_back_button.borrow();
                    if let Some(go_back_button) = go_back_button.as_ref() {
                        go_back_button.set_visible(true);
                    }
                }
            });

            self.row.add_controller(click);
        }
    }

    impl WidgetImpl for File {}
    impl BoxImpl for File {}

    impl File {
        pub fn set_files(&self, files: ListBox) {
            self.files.replace(Some(files));
        }

        pub fn set_go_back_button(&self, go_back_button: Option<gtk::Button>) {
            *self.go_back_button.borrow_mut() = go_back_button;
        }

        pub fn set_path(&self, path: String) {
            *self.path.borrow_mut() = path;
        }
        
        pub fn set_current_path(&self, path: Option<String>) {
            *self.current_path.borrow_mut() = path;
        }

        pub fn set_is_dir(&self, is_dir: bool) {
            self.is_dir.set(is_dir);
            match is_dir {
                true => self.file_icon.set_icon_name(Some("directory-symbolic")),
                false => self.file_icon.set_icon_name(Some("music-alt-symbolic")),
            };
        }

    }
}

glib::wrapper! {
  pub struct File(ObjectSubclass<imp::File>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl File {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_files(&self, path: Option<String>) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response_ = rt.block_on(async {
            let url = build_url();
            let mut client = BrowseServiceClient::connect(url).await?;
            let response = client
                .tree_get_entries(TreeGetEntriesRequest {
                    path
                })
                .await?
                .into_inner();
            Ok::<TreeGetEntriesResponse, Error>(response)
        });

        if let Ok(response) = response_ {
            let files = self.imp().files.borrow();
            let files_ref = files.as_ref();
            let files_ref = files_ref.unwrap();
                    
            while let Some(file) = files_ref.first_child() {
                files_ref.remove(&file);
            }
                
            for entry in response.entries {
                let file = File::new();
                let filename = entry.name.split("/").last().unwrap();
                file.imp().set_files(files_ref.clone());
                file.imp().set_go_back_button(self.imp().go_back_button.borrow().clone());
                file.imp().file_name.set_text(filename);
                file.imp().set_path(entry.name.clone());
                file.imp().set_is_dir(entry.attr == 16);
                files_ref.append(&file);
            }
        }
    }
    
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or("localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or("6061".to_string());
    format!("tcp://{}:{}", host, port)
    
}
