use crate::api::news::NewsArticle;
use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MAX_CACHE: usize = 200;

pub struct NewsCache {
    conn: Connection,
}

impl NewsCache {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS news (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                source TEXT NOT NULL,
                published_at INTEGER NOT NULL,
                description TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn load_latest(&self, limit: usize) -> Result<Vec<NewsArticle>> {
        let mut stmt = self.conn.prepare(
            "SELECT title, source, published_at, description
             FROM news
             ORDER BY published_at DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(NewsArticle {
                title: row.get(0)?,
                source: row.get(1)?,
                published_at: row.get(2)?,
                link: None,
                description: row.get(3)?,
            })
        })?;

        let mut articles = Vec::new();
        for row in rows {
            articles.push(row?);
        }
        Ok(articles)
    }

    pub fn save_articles(&mut self, articles: &[NewsArticle]) -> Result<Vec<NewsArticle>> {
        // Load existing cached items
        let mut merged = self.load_latest(MAX_CACHE).unwrap_or_default();
        merged.extend_from_slice(articles);

        // Sort by published_at desc and deduplicate by (title, source, published_at)
        merged.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        let mut seen = HashSet::new();
        merged.retain(|a| {
            let title_hash = hash_title(&a.title);
            let key = a
                .link
                .as_ref()
                .map(|l| format!("{}::{}", l, title_hash))
                .unwrap_or_else(|| format!("{}::{}::{}", title_hash, a.source, a.published_at));
            seen.insert(key)
        });

        merged.truncate(MAX_CACHE);

        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM news", [])?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO news (title, source, published_at, description)
                 VALUES (?1, ?2, ?3, ?4)",
            )?;
            for article in &merged {
                stmt.execute(params![
                    &article.title,
                    &article.source,
                    article.published_at,
                    &article.description
                ])?;
            }
        }

        tx.commit()?;
        Ok(merged)
    }
}

fn hash_title(title: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    title.hash(&mut hasher);
    hasher.finish()
}
