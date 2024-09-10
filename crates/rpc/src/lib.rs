pub mod browse;
pub mod metadata;
pub mod playback;
pub mod playlist;
pub mod server;
pub mod settings;
pub mod sound;
pub mod tagcache;

pub mod api {
    #[path = ""]
    pub mod rockbox {
        #[path = "rockbox.v1alpha1.rs"]
        pub mod v1alpha1;

        pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("api/rockbox_descriptor.bin");
    }
}
