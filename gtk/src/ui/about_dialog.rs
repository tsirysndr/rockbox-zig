use adw::prelude::*;

use crate::ui::window::RbApplicationWindow;

pub fn show(parent: &RbApplicationWindow) {
    let about = adw::AboutDialog::from_appdata("/mg/tsirysndr/Rockbox/metainfo.xml", Some(""));
    about.set_designers(&["Tsiry Sandratraina"]);
    about.set_developers(&["Tsiry Sandratraina https://github.com/tsirysndr"]);
    about.set_copyright("© 2024 Tsiry Sandratraina");
    about.present(Some(parent));
}
