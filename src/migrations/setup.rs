use sea_orm::{sea_query::Index, ConnectionTrait, DbBackend, DbConn, DbErr, ExecResult, Schema};

use crate::entities::session;
use crate::entities::user;

pub async fn up(conn: &DbConn) -> anyhow::Result<Vec<ExecResult>, DbErr> {
    let schema = Schema::new(DbBackend::Sqlite);
    let mut user_table = schema.create_table_from_entity(user::Entity);
    let mut username_index = Index::create()
        .name("idx-user-username")
        .table(user::Entity)
        .col(user::Column::Username)
        .to_owned();

    let mut results = vec![];
    results.push(
        conn.execute(
            conn.get_database_backend()
                .build(user_table.if_not_exists()),
        )
        .await?,
    );
    results.push(
        conn.execute(
            conn.get_database_backend()
                .build(username_index.if_not_exists()),
        )
        .await?,
    );

    let mut session_table = schema.create_table_from_entity(session::Entity);
    results.push(
        conn.execute(
            conn.get_database_backend()
                .build(session_table.if_not_exists()),
        )
        .await?,
    );

    Ok(results)
}
