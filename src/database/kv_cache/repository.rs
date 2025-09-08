use crate::database::columns::CacheTypeColumn;
use crate::database::kv_cache::{model, KvCache, KvCacheTable};
use crate::global::client::db::get_db;
use crate::util::structs::cache_type::CacheType;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, EntityTrait, QueryFilter,
};
use serde_json::Value as JsonValue;

pub struct KvCacheRepository;

impl KvCacheRepository {
    pub async fn get(cache_type: CacheType, key: &str) -> Result<Option<KvCache>> {
        let db = get_db().await;

        let now = Utc::now();
        let cache_type_column = CacheTypeColumn::from(cache_type);
        let result = KvCacheTable::find()
            .filter(model::Column::Type.eq(cache_type_column))
            .filter(model::Column::Key.eq(key))
            .filter(model::Column::ValidUntil.gt(now))
            .one(db)
            .await?;

        Ok(result)
    }

    pub async fn put(
        cache_type: CacheType,
        key: String,
        value: JsonValue,
        valid_until: DateTime<Utc>,
    ) -> Result<()> {
        let db = get_db().await;

        let active_model = model::ActiveModel {
            r#type: Set(CacheTypeColumn::from(cache_type)),
            key: Set(key),
            value: Set(value),
            valid_until: Set(valid_until),
            created_at: NotSet,
            updated_at: NotSet,
        };

        KvCacheTable::insert(active_model)
            .on_conflict(
                OnConflict::columns([model::Column::Type, model::Column::Key])
                    .update_columns([model::Column::Value, model::Column::ValidUntil])
                    .to_owned(),
            )
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn evict(cache_type: CacheType, key: &str) -> Result<()> {
        let db = get_db().await;

        let cache_type_column = CacheTypeColumn::from(cache_type);
        KvCacheTable::delete_many()
            .filter(model::Column::Type.eq(cache_type_column))
            .filter(model::Column::Key.eq(key))
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn cleanup_expired() -> Result<u64> {
        let db = get_db().await;

        let now = Utc::now();
        let result = KvCacheTable::delete_many()
            .filter(model::Column::ValidUntil.lt(now))
            .exec(db)
            .await?;

        Ok(result.rows_affected)
    }
}
