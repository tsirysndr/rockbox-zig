#[derive(sqlx::FromRow, Default)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}
