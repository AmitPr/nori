use axum::http::HeaderValue;
use sea_orm::{entity::prelude::*, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(column_name = "user_id")]
    pub user_id: i32,
    #[sea_orm(column_name = "expires_at")]
    pub expires_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    User,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::User => Entity::belongs_to(super::user::Entity)
                .from(Column::UserId)
                .to(super::user::Column::Id)
                .into(),
        }
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn create_session(user_id: i32, expires_at: Option<DateTimeUtc>) -> ActiveModel {
        Self {
            id: Set(Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            expires_at: Set(expires_at),
        }
    }

    pub fn get_cookie(&self) -> HeaderValue {
        let header = HeaderValue::from_str(
            format!(
                "session={}; Secure; HttpOnly; SameSite=Strict; {}",
                self.id.as_ref(),
                if self.expires_at.as_ref().is_none() {
                    "Expires=Thu, 31 Dec 2037 23:55:55 GMT;".to_string()
                } else {
                    format!(
                        "Expires={};",
                        self.expires_at
                            .as_ref()
                            .unwrap()
                            .format("%a, %d %b %Y %T GMT")
                            .to_string()
                    )
                }
            )
            .as_str(),
        )
        .expect("could not create header");
        header
    }
}
