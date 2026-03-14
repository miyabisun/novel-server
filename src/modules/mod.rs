pub mod kakuyomu;
pub mod syosetu;

use crate::error::AppError;
use serde_json::Value;

/// Site type enum dispatch — simpler and more type-safe than trait objects.
/// Each method's match arm delegates to a site-specific module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
    Narou,
    Nocturne,
    Kakuyomu,
}

impl ModuleType {
    pub fn resolve(s: &str) -> Result<Self, AppError> {
        match s {
            "narou" => Ok(Self::Narou),
            "nocturne" => Ok(Self::Nocturne),
            "kakuyomu" => Ok(Self::Kakuyomu),
            _ => Err(AppError::BadRequest("Invalid type".to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Narou => "narou",
            Self::Nocturne => "nocturne",
            Self::Kakuyomu => "kakuyomu",
        }
    }

    pub async fn fetch_ranking_list(
        &self,
        client: &reqwest::Client,
        limit: usize,
        period: &str,
    ) -> Result<Value, AppError> {
        match self {
            Self::Narou => {
                syosetu::fetch_ranking_list(&syosetu::NAROU, client, limit, period).await
            }
            Self::Nocturne => {
                syosetu::fetch_ranking_list(&syosetu::NOCTURNE, client, limit, period).await
            }
            Self::Kakuyomu => kakuyomu::fetch_ranking_list(client, period).await,
        }
    }

    pub async fn fetch_page(
        &self,
        client: &reqwest::Client,
        id: &str,
        page_id: &str,
    ) -> Result<Option<String>, AppError> {
        match self {
            Self::Narou => syosetu::fetch_page(&syosetu::NAROU, client, id, page_id).await,
            Self::Nocturne => syosetu::fetch_page(&syosetu::NOCTURNE, client, id, page_id).await,
            Self::Kakuyomu => kakuyomu::fetch_page(client, id, page_id).await,
        }
    }

    pub async fn fetch_detail(
        &self,
        client: &reqwest::Client,
        id: &str,
    ) -> Result<Value, AppError> {
        match self {
            Self::Narou => syosetu::fetch_detail(&syosetu::NAROU, client, id).await,
            Self::Nocturne => syosetu::fetch_detail(&syosetu::NOCTURNE, client, id).await,
            Self::Kakuyomu => kakuyomu::fetch_detail(client, id).await,
        }
    }

    pub async fn fetch_search(
        &self,
        client: &reqwest::Client,
        word: &str,
    ) -> Result<Value, AppError> {
        match self {
            Self::Narou => syosetu::fetch_search(&syosetu::NAROU, client, word).await,
            Self::Nocturne => syosetu::fetch_search(&syosetu::NOCTURNE, client, word).await,
            Self::Kakuyomu => kakuyomu::fetch_search(client, word).await,
        }
    }

    pub async fn fetch_toc(&self, client: &reqwest::Client, id: &str) -> Result<Value, AppError> {
        match self {
            Self::Narou => syosetu::fetch_toc(&syosetu::NAROU, client, id).await,
            Self::Nocturne => syosetu::fetch_toc(&syosetu::NOCTURNE, client, id).await,
            Self::Kakuyomu => kakuyomu::fetch_toc(client, id).await,
        }
    }

    pub async fn fetch_data(
        &self,
        client: &reqwest::Client,
        ids: &[String],
    ) -> Result<Vec<Value>, AppError> {
        match self {
            Self::Narou => syosetu::fetch_data(&syosetu::NAROU, client, ids).await,
            Self::Nocturne => syosetu::fetch_data(&syosetu::NOCTURNE, client, ids).await,
            Self::Kakuyomu => kakuyomu::fetch_data(client, ids).await,
        }
    }

    pub async fn fetch_datum(&self, client: &reqwest::Client, id: &str) -> Result<Value, AppError> {
        match self {
            Self::Narou => syosetu::fetch_datum(&syosetu::NAROU, client, id).await,
            Self::Nocturne => syosetu::fetch_datum(&syosetu::NOCTURNE, client, id).await,
            Self::Kakuyomu => kakuyomu::fetch_datum(client, id).await,
        }
    }
}
