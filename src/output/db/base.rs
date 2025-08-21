use error_stack::{Result, ResultExt};
use modql::{SIden, field::HasSeaFields};
use sea_query::{
    Alias, Asterisk, Expr, Iden, IntoIden, OnConflict, PostgresQueryBuilder, Query, TableRef,
};
use sea_query_binder::SqlxBinder;
use sqlx::{FromRow, Postgres, Transaction};

use crate::errors::Error;

pub trait Dao {
    const TABLE: &'static str;
    fn table_ref() -> TableRef {
        TableRef::Table(SIden(Self::TABLE).into_iden())
    }
}

#[derive(Iden)]
pub enum CommonIden {
    Id,
}

pub async fn dao_create<D, E>(tx: &mut Transaction<'_, Postgres>, req: E) -> Result<i64, Error>
where
    E: HasSeaFields,
    D: Dao,
{
    let fields = req.not_none_sea_fields();
    let (columns, sea_values) = fields.for_sea_insert();
    let mut query = Query::insert();
    query
        .into_table(D::table_ref())
        .columns(columns)
        .values(sea_values)
        .change_context_lazy(|| Error::Message("failed to create record".to_string()))?
        .returning(Query::returning().columns([CommonIden::Id]));
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    log::debug!("sql: {} values: {:?}", sql, values);
    let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
    let (id,) = sqlx_query
        .fetch_one(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to create record".to_string()))?;
    Ok(id)
}

pub async fn dao_first<D, T>(tx: &mut Transaction<'_, Postgres>) -> Result<Option<T>, Error>
where
    D: Dao,
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Unpin + Send,
{
    let (sql, values) = Query::select()
        .from(D::table_ref())
        .column(Asterisk)
        .build_sqlx(PostgresQueryBuilder);

    log::debug!("sql: {} values: {:?}", sql, values);
    let ret = sqlx::query_as_with::<_, T, _>(&sql, values)
        .fetch_optional(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to fetch one record".to_string()))?;
    Ok(ret)
}

pub async fn dao_upsert<D, E>(
    tx: &mut Transaction<'_, Postgres>,
    req: E,
    conflict_column: &str,
    update_columns: &[&str],
) -> Result<i64, Error>
where
    E: HasSeaFields,
    D: Dao,
{
    let fields = req.not_none_sea_fields();
    let (columns, sea_values) = fields.for_sea_insert();

    let mut query = Query::insert();
    query
        .into_table(D::table_ref())
        .columns(columns)
        .values(sea_values)
        .change_context_lazy(|| Error::Message("failed to upsert record".to_string()))?;

    let on_conflict = if update_columns.is_empty() {
        OnConflict::column(Alias::new(conflict_column))
            .do_nothing()
            .to_owned()
    } else {
        let mut on_conflict = OnConflict::column(Alias::new(conflict_column));
        for &col in update_columns {
            on_conflict = on_conflict.update_column(Alias::new(col)).to_owned();
        }
        on_conflict
    };

    query.on_conflict(on_conflict);
    query.returning(Query::returning().columns([CommonIden::Id]));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    log::debug!("sql: {} values: {:?}", sql, values);

    let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
    let (id,) = sqlx_query
        .fetch_one(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to upsert record".to_string()))?;

    Ok(id)
}

pub async fn dao_fetch_by_column<D, T>(
    tx: &mut Transaction<'_, Postgres>,
    column_name: &str,
    value: &str,
) -> Result<Option<T>, Error>
where
    D: Dao,
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Unpin + Send,
{
    let (sql, values) = Query::select()
        .from(D::table_ref())
        .column(Asterisk)
        .and_where(Expr::col(Alias::new(column_name)).eq(value))
        .build_sqlx(PostgresQueryBuilder);

    log::debug!("sql: {} values: {:?}", sql, values);

    let result = sqlx::query_as_with::<_, T, _>(&sql, values)
        .fetch_optional(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to fetch record by column".to_string()))?;

    Ok(result)
}
pub async fn dao_update<D, E>(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    req: E,
) -> Result<u64, Error>
where
    E: HasSeaFields,
    D: Dao,
{
    let fields = req.not_none_sea_fields();
    let (columns, sea_values) = fields.for_sea_insert();

    let mut query = Query::update();
    query
        .table(D::table_ref())
        .and_where(Expr::col(CommonIden::Id).eq(id));

    // Add values to update
    for (column, value) in columns.into_iter().zip(sea_values.into_iter()) {
        query.value(column, value);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    log::debug!("sql: {} values: {:?}", sql, values);

    let sqlx_query = sqlx::query_with(&sql, values);
    let result = sqlx_query
        .execute(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to update record".to_string()))?;

    Ok(result.rows_affected())
}
