use async_graphql::*;

#[derive(Default)]
pub struct BrowseQuery;

#[Object]
impl BrowseQuery {
    async fn browse_id3(&self) -> String {
        "browse id3".to_string()
    }

    async fn tree_get_context(&self) -> String {
        "tree get context".to_string()
    }

    async fn tree_get_entry_at(&self) -> String {
        "tree get entry at".to_string()
    }

    async fn rockbox_browse(&self) -> String {
        "rockbox browse".to_string()
    }
}
