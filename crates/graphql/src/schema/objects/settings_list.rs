use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct SettingsList {
    pub flags: u32,
    // pub setting: *mut c_void,
    pub lang_id: i32,
    // pub default_val: StorageType, // union storage_type
    pub cfg_name: String, // const char*
    pub cfg_vals: String, // const char*

                          // union with different possible struct types
                          // pub setting_type: SettingsTypeUnion,
}

#[Object]
impl SettingsList {
    async fn flags(&self) -> u32 {
        self.flags
    }

    // async fn setting(&self) -> *mut c_void {
    //   self.setting
    // }

    async fn lang_id(&self) -> i32 {
        self.lang_id
    }

    // async fn default_val(&self) -> StorageType {
    //   self.default_val
    // }

    async fn cfg_name(&self) -> &str {
        &self.cfg_name
    }

    async fn cfg_vals(&self) -> &str {
        &self.cfg_vals
    }

    // async fn setting_type(&self) -> SettingsTypeUnion {
    //   self.setting_type
    // }
}
