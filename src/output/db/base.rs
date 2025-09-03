use error_stack::ResultExt;
use modql::{SIden, field::HasSeaFields};
use sea_query::{
    Alias, Asterisk, Condition, Expr, Iden, IntoIden, OnConflict, PostgresQueryBuilder, Query,
    SelectStatement, TableRef,
};
use sea_query_binder::SqlxBinder;
use sqlx::{FromRow, Postgres, Row, Transaction};

use crate::{AppResult, errors::Error};

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

pub async fn dao_create<D, E>(tx: &mut Transaction<'_, Postgres>, req: E) -> AppResult<i64>
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

pub async fn dao_first<D, T>(tx: &mut Transaction<'_, Postgres>) -> AppResult<Option<T>>
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
) -> AppResult<i64>
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
) -> AppResult<Option<T>>
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
pub async fn dao_update<D, E>(tx: &mut Transaction<'_, Postgres>, id: i64, req: E) -> AppResult<u64>
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

pub async fn dao_update_field<D>(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    field_name: &str,
    field_value: impl Into<sea_query::Value>,
) -> AppResult<u64>
where
    D: Dao,
{
    let mut query = Query::update();
    query
        .table(D::table_ref())
        .and_where(Expr::col(CommonIden::Id).eq(id))
        .value(Alias::new(field_name), field_value);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    log::debug!("sql: {} values: {:?}", sql, values);

    let sqlx_query = sqlx::query_with(&sql, values);
    let result = sqlx_query
        .execute(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to update field".to_string()))?;

    Ok(result.rows_affected())
}

pub async fn dao_fetch_by_id<D, T>(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
) -> AppResult<Option<T>>
where
    D: Dao,
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Unpin + Send,
{
    let (sql, values) = Query::select()
        .from(D::table_ref())
        .column(Asterisk)
        .and_where(Expr::col(CommonIden::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    log::debug!("sql: {} values: {:?}", sql, values);

    let result = sqlx::query_as_with::<_, T, _>(&sql, values)
        .fetch_optional(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to fetch record by id".to_string()))?;

    Ok(result)
}

pub struct DaoQueryBuilder<D: Dao> {
    query: SelectStatement,
    conditions: Vec<Condition>,
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Dao> Default for DaoQueryBuilder<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D: Dao> DaoQueryBuilder<D> {
    pub fn new() -> Self {
        let mut query = Query::select();
        query.from(D::table_ref()).column(Asterisk);

        Self {
            query,
            conditions: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn and_where_like(mut self, column: &str, value: &str) -> Self {
        let condition = Expr::col(Alias::new(column)).like(format!("%{}%", value));
        self.query.and_where(condition.clone());
        self.conditions.push(Condition::all().add(condition));
        self
    }

    pub fn and_where_eq(mut self, column: &str, value: i64) -> Self {
        let condition = Expr::col(Alias::new(column)).eq(value);
        self.query.and_where(condition.clone());
        self.conditions.push(Condition::all().add(condition));
        self
    }

    pub fn and_where_in(mut self, column: &str, values: &[i64]) -> Self {
        if !values.is_empty() {
            let condition = Expr::col(Alias::new(column)).is_in(values.iter().copied());
            self.query.and_where(condition.clone());
            self.conditions.push(Condition::all().add(condition));
        }
        self
    }

    pub fn and_where_bool(mut self, column: &str, value: bool) -> Self {
        let condition = Expr::col(Alias::new(column)).eq(value);
        self.query.and_where(condition.clone());
        self.conditions.push(Condition::all().add(condition));
        self
    }

    pub fn order_by_desc(mut self, column: &str) -> Self {
        self.query
            .order_by(Alias::new(column), sea_query::Order::Desc);
        self
    }

    pub fn limit_offset(mut self, limit: i64, offset: i64) -> Self {
        self.query.limit(limit as u64).offset(offset as u64);
        self
    }

    pub async fn fetch_all<T>(self, tx: &mut Transaction<'_, Postgres>) -> AppResult<Vec<T>>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Unpin + Send,
    {
        let (sql, values) = self.query.build_sqlx(PostgresQueryBuilder);
        log::debug!("sql: {} values: {:?}", sql, values);

        let result = sqlx::query_as_with::<_, T, _>(&sql, values)
            .fetch_all(tx.as_mut())
            .await
            .change_context_lazy(|| Error::Message("failed to fetch records".to_string()))?;

        Ok(result)
    }

    pub async fn count(self, tx: &mut Transaction<'_, Postgres>) -> AppResult<i64> {
        let mut count_query = Query::select();
        count_query
            .from(D::table_ref())
            .expr(Expr::col(CommonIden::Id).count());

        for condition in self.conditions {
            count_query.cond_where(condition);
        }

        let (sql, values) = count_query.build_sqlx(PostgresQueryBuilder);
        log::debug!("sql: {} values: {:?}", sql, values);

        let result = sqlx::query_with(&sql, values)
            .fetch_one(tx.as_mut())
            .await
            .change_context_lazy(|| Error::Message("failed to count records".to_string()))?;

        Ok(result.get::<i64, _>(0))
    }
}
