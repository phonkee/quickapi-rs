#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct SimpleUser {
    pub id: i32,
    pub username: String,
}

impl From<entity::UserModel> for SimpleUser {
    fn from(user: entity::UserModel) -> Self {
        SimpleUser {
            id: user.id,
            username: user.username,
        }
    }
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct UserIdOnly {
    pub id: i32,
}

impl From<entity::UserModel> for UserIdOnly {
    fn from(user: entity::UserModel) -> Self {
        UserIdOnly { id: user.id }
    }
}
