//! PagingRequest → SeaORM select assembly — the
//! paging/filter pipeline: the `query` JSON filter syntax
//! (rust-utils query_parser, `__`-suffixed operators, snake-cased
//! fields), the `orderBy` JSON-array string, the page/offset/none paging
//! modes. PG lacks ILIKE in sea-query → case-insensitive matching goes
//! through `lower(x) LIKE lower(y)`.

use sea_orm::sea_query::{Alias, BinOper, Condition, Expr, ExprTrait, Func, SimpleExpr};
use sea_orm::Value as QValue;
use sea_orm::{DatabaseConnection, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Select};

use proto::proto::pagination::paging_request::FilteringType;
use proto::proto::pagination::PagingRequest;

use crate::db_err;

/// The value-binding kind of a filter column (filter values arrive as
/// JSON strings; PG needs matching literal types).
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Number,
    Bool,
    Text,
}

/// The value-binding kind of one entity column, read off the entity's
/// own schema at runtime — a column absent from the entity (or an
/// entity-less name) binds as text. Integer-family columns bind
/// numeric, `Boolean` binds bool.
fn column_kind<E>(field: &str) -> Kind
where
    E: sea_orm::EntityTrait,
{
    use sea_orm::entity::ColumnTrait as _;
    let mut kind = Kind::Text;
    for col in <E::Column as sea_orm::Iterable>::iter() {
        if sea_orm::sea_query::Iden::to_string(&col) == field {
            kind = match col.def().get_column_type() {
                sea_orm::sea_query::ColumnType::TinyInteger
                | sea_orm::sea_query::ColumnType::SmallInteger
                | sea_orm::sea_query::ColumnType::Integer
                | sea_orm::sea_query::ColumnType::BigInteger
                | sea_orm::sea_query::ColumnType::TinyUnsigned
                | sea_orm::sea_query::ColumnType::SmallUnsigned
                | sea_orm::sea_query::ColumnType::Unsigned
                | sea_orm::sea_query::ColumnType::BigUnsigned => Kind::Number,
                sea_orm::sea_query::ColumnType::Boolean => Kind::Bool,
                _ => Kind::Text,
            };
            break;
        }
    }
    kind
}

fn value_of(kind: Kind, value: &str) -> QValue {
    match kind {
        Kind::Number => value
            .parse::<i64>()
            .map(|n| QValue::BigInt(Some(n)))
            .unwrap_or_else(|_| QValue::String(Some(value.to_string()))),
        Kind::Bool => QValue::Bool(Some(value == "true")),
        Kind::Text => QValue::String(Some(value.to_string())),
    }
}

fn cmp(field: &str, op: BinOper, value: &str, kind: Kind) -> SimpleExpr {
    Expr::col(Alias::new(field)).binary(op, SimpleExpr::Value(value_of(kind, value)))
}

fn lower_like(field: &str, pattern: &str) -> SimpleExpr {
    let col = Func::cust("lower").arg(Expr::col(Alias::new(field)));
    let pat = Func::cust("lower").arg(SimpleExpr::Value(QValue::String(Some(pattern.to_string()))));
    col.binary(BinOper::Like, pat)
}

fn split_values(value: &str) -> Vec<String> {
    rust_utils::query_parser::split_query_values(value)
        .into_iter()
        .map(String::from)
        .collect()
}

fn in_list(field: &str, values: &[String], kind: Kind) -> Option<SimpleExpr> {
    match values.len() {
        0 => None,
        1 => Some(cmp(field, BinOper::Equal, &values[0], kind)),
        _ => Some(
            Expr::col(Alias::new(field)).binary(
                BinOper::In,
                SimpleExpr::Tuple(
                    values
                        .iter()
                        .map(|v| SimpleExpr::Value(value_of(kind, v)))
                        .collect(),
                ),
            ),
        ),
    }
}

fn filter_condition(kind: Kind, field: &str, op: &str, value: &str) -> Option<Condition> {
    use rust_utils::query_parser as qp;
    let cond = match op {
        "" | qp::FILTER_EXACT => Condition::all().add(cmp(field, BinOper::Equal, value, kind)),
        qp::FILTER_NOT | qp::FILTER_NOT_IN => {
            let expr = in_list(field, &split_values(value), kind)?;
            Condition::all().add(Expr::not(expr))
        }
        qp::FILTER_IN => Condition::all().add(in_list(field, &split_values(value), kind)?),
        qp::FILTER_GT => Condition::all().add(cmp(field, BinOper::GreaterThan, value, kind)),
        qp::FILTER_GTE => {
            Condition::all().add(cmp(field, BinOper::GreaterThanOrEqual, value, kind))
        }
        qp::FILTER_LT => Condition::all().add(cmp(field, BinOper::SmallerThan, value, kind)),
        qp::FILTER_LTE => {
            Condition::all().add(cmp(field, BinOper::SmallerThanOrEqual, value, kind))
        }
        qp::FILTER_RANGE => {
            let values = split_values(value);
            if values.len() != 2 {
                return None;
            }
            Condition::all()
                .add(cmp(field, BinOper::GreaterThanOrEqual, &values[0], kind))
                .add(cmp(field, BinOper::SmallerThanOrEqual, &values[1], kind))
        }
        qp::FILTER_IS_NULL => Condition::all().add(Expr::col(Alias::new(field)).is_null()),
        qp::FILTER_NOT_IS_NULL => Condition::all().add(Expr::col(Alias::new(field)).is_not_null()),
        qp::FILTER_CONTAINS => {
            Condition::all().add(Expr::col(Alias::new(field)).like(format!("%{value}%")))
        }
        qp::FILTER_ICONTAINS => Condition::all().add(lower_like(field, &format!("%{value}%"))),
        qp::FILTER_STARTS_WITH => {
            Condition::all().add(Expr::col(Alias::new(field)).like(format!("{value}%")))
        }
        qp::FILTER_ISTARTSWITH => Condition::all().add(lower_like(field, &format!("{value}%"))),
        qp::FILTER_ENDS_WITH => {
            Condition::all().add(Expr::col(Alias::new(field)).like(format!("%{value}")))
        }
        qp::FILTER_IENDSWITH => Condition::all().add(lower_like(field, &format!("%{value}"))),
        qp::FILTER_IEXACT => Condition::all().add(lower_like(field, value)),
        // The gorm layer silently drops regex/search (operator-matrix D1/D2).
        qp::FILTER_REGEX | qp::FILTER_IREGEX | qp::FILTER_SEARCH => return None,
        _ => return None,
    };
    Some(cond)
}

/// The paged fetch envelope shared by every repository listing: rows of
/// the assembled select plus the total matching the same base (the row
/// count itself under `no_paging`). `base` carries the fixed predicates
/// and ordering; request-side filtering/ordering/slicing rides [`apply`].
pub async fn fetch_paged<E>(
    db: &DatabaseConnection,
    base: Select<E>,
    req: &PagingRequest,
) -> Result<(Vec<E::Model>, u64), rushwind_http_binding::envelope::StatusError>
where
    E: sea_orm::EntityTrait,
    E::Model: sea_orm::FromQueryResult + Send + Sync + 'static,
{
    let no_paging = req.no_paging.unwrap_or(false);
    let total = if no_paging {
        None
    } else {
        // count() clears ordering internally, so a pre-sorted base is safe.
        Some(base.clone().count(db).await.map_err(db_err)?)
    };
    let paged = apply(base, req);
    let rows = paged.all(db).await.map_err(db_err)?;
    let total = total.unwrap_or(rows.len() as u64);
    Ok((rows, total))
}

/// Applies the PagingRequest to a select: filter conditions, ordering
/// (falling back to `id`) and page slicing.
fn apply<E>(mut select: Select<E>, req: &PagingRequest) -> Select<E>
where
    E: sea_orm::EntityTrait,
{
    if let Some(FilteringType::Query(query)) = &req.filtering_type {
        let mut conditions: Vec<Condition> = Vec::new();
        let _ = rust_utils::query_parser::parse_filter_json_string(query, |field, op, value| {
            if let Some(cond) = filter_condition(column_kind::<E>(field), field, op, value) {
                conditions.push(cond);
            }
        });
        for cond in conditions {
            select = select.filter(cond);
        }
    }

    let mut order_specs: Vec<(String, bool)> = Vec::new();
    if let Some(order_by) = &req.order_by {
        if let Ok(fields) = serde_json::from_str::<Vec<String>>(order_by) {
            for field in fields {
                rust_utils::query_parser::parse_order_by_field(&field, |f, desc| {
                    order_specs.push((f.to_string(), desc));
                });
            }
        } else {
            let _ = rust_utils::query_parser::parse_order_by_string(order_by, |f, desc| {
                order_specs.push((f.to_string(), desc));
            });
        }
    }
    for sorting in &req.sorting {
        if sorting.field.is_empty() {
            continue;
        }
        order_specs.push((
            rust_utils::stringcase::to_snake_case(&sorting.field),
            sorting.direction == 1, // Direction::DESC
        ));
    }
    if order_specs.is_empty() {
        order_specs.push(("id".to_string(), false));
    }
    for (field, desc) in &order_specs {
        let col = Expr::col(Alias::new(field));
        select = if *desc {
            select.order_by(col, sea_orm::Order::Desc)
        } else {
            select.order_by(col, sea_orm::Order::Asc)
        };
    }

    let no_paging = req.no_paging.unwrap_or(false);
    let page = std::cmp::max(req.page.unwrap_or(1) as u64, 1);
    let page_size = std::cmp::max(req.page_size.unwrap_or(10) as u64, 1);
    let (offset, limit) = if no_paging {
        (0, u64::MAX)
    } else if req.offset.is_some() || req.limit.is_some() {
        (
            req.offset.unwrap_or(0),
            std::cmp::max(req.limit.unwrap_or(10) as u64, 1),
        )
    } else {
        ((page - 1) * page_size, page_size)
    };
    if !no_paging {
        select = select.offset(offset).limit(limit);
    }

    select
}
