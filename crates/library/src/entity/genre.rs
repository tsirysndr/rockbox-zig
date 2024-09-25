#[derive(sqlx::FromRow, Default)]
pub struct Genre {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
}
