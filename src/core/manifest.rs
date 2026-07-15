use rusqlite::Connection;

use super::{db_dir, CoreError, Result};

/// 安装记录，对应 SQLite 表的一行
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstallRecord {
    /// 唯一 ID（UUID v4）
    pub id: String,
    /// 包名（如 python3.7, node18）
    pub name: String,
    /// 版本号
    pub version: Option<String>,
    /// 安装路径（如 ~/.oneinit/envs/python3.7）
    pub install_path: String,
    /// 原始下载 URL
    pub archive_url: Option<String>,
    /// SHA256 校验值
    pub sha256: Option<String>,
    /// 写入 PATH 的条目列表（JSON 数组）
    pub path_entries: Vec<String>,
    /// 生成的配置文件列表（JSON 数组）
    pub config_files: Vec<String>,
    /// 安装时间（ISO8601）
    pub installed_at: String,
    /// 安装前 PATH 备份（用于卸载回滚）
    pub original_path: Option<String>,
    /// 环境变量备份（JSON 对象）
    pub env_vars_backup: serde_json::Value,
}

/// 清单系统 — 管理所有工具的安装记录
pub struct Manifest {
    db: Connection,
}

impl Manifest {
    /// 打开（或创建）清单数据库
    pub fn open() -> Result<Self> {
        let db_dir = db_dir();
        std::fs::create_dir_all(&db_dir)?;

        let db_path = db_dir.join("oneinit.db");
        let db = Connection::open(&db_path)?;

        // 启用 WAL 模式提升并发性能
        db.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;

        let manifest = Self { db };
        manifest.init_table()?;

        Ok(manifest)
    }

    /// 初始化数据库表
    fn init_table(&self) -> Result<()> {
        self.db.execute_batch(
            "CREATE TABLE IF NOT EXISTS installed (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL UNIQUE,
                version         TEXT,
                install_path    TEXT NOT NULL,
                archive_url     TEXT,
                sha256          TEXT,
                path_entries    TEXT NOT NULL DEFAULT '[]',
                config_files    TEXT NOT NULL DEFAULT '[]',
                installed_at    TEXT NOT NULL,
                original_path   TEXT,
                env_vars_backup TEXT NOT NULL DEFAULT '{}'
            );",
        )?;
        Ok(())
    }

    /// 添加安装记录
    pub fn add(&self, record: &InstallRecord) -> Result<String> {
        self.db.execute(
            "INSERT INTO installed
             (id, name, version, install_path, archive_url, sha256,
              path_entries, config_files, installed_at, original_path, env_vars_backup)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                record.id,
                record.name,
                record.version,
                record.install_path,
                record.archive_url,
                record.sha256,
                serde_json::to_string(&record.path_entries).unwrap_or_default(),
                serde_json::to_string(&record.config_files).unwrap_or_default(),
                record.installed_at,
                record.original_path,
                serde_json::to_string(&record.env_vars_backup).unwrap_or_default(),
            ],
        )?;
        Ok(record.id.clone())
    }

    /// 查询安装记录（按包名）
    pub fn get(&self, name: &str) -> Result<Option<InstallRecord>> {
        let mut stmt = self.db.prepare(
            "SELECT id, name, version, install_path, archive_url, sha256,
                    path_entries, config_files, installed_at, original_path, env_vars_backup
             FROM installed WHERE name = ?1",
        )?;

        let result = stmt.query_row(rusqlite::params![name], |row| {
            Ok(InstallRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                install_path: row.get(3)?,
                archive_url: row.get(4)?,
                sha256: row.get(5)?,
                path_entries: serde_json::from_str(row.get::<_, String>(6)?.as_str()).unwrap_or_default(),
                config_files: serde_json::from_str(row.get::<_, String>(7)?.as_str()).unwrap_or_default(),
                installed_at: row.get(8)?,
                original_path: row.get(9)?,
                env_vars_backup: serde_json::from_str(row.get::<_, String>(10)?.as_str())
                    .unwrap_or(serde_json::json!({})),
            })
        });

        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(CoreError::Database(e.to_string())),
        }
    }

    /// 列出所有安装记录
    pub fn list(&self) -> Result<Vec<InstallRecord>> {
        let mut stmt = self.db.prepare(
            "SELECT id, name, version, install_path, archive_url, sha256,
                    path_entries, config_files, installed_at, original_path, env_vars_backup
             FROM installed ORDER BY installed_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(InstallRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                install_path: row.get(3)?,
                archive_url: row.get(4)?,
                sha256: row.get(5)?,
                path_entries: serde_json::from_str(row.get::<_, String>(6)?.as_str()).unwrap_or_default(),
                config_files: serde_json::from_str(row.get::<_, String>(7)?.as_str()).unwrap_or_default(),
                installed_at: row.get(8)?,
                original_path: row.get(9)?,
                env_vars_backup: serde_json::from_str(row.get::<_, String>(10)?.as_str())
                    .unwrap_or(serde_json::json!({})),
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    /// 删除安装记录（卸载时调用，返回记录用于回滚）
    pub fn remove(&self, name: &str) -> Result<Option<InstallRecord>> {
        let record = self.get(name)?;
        if record.is_none() {
            return Ok(None);
        }

        self.db
            .execute("DELETE FROM installed WHERE name = ?1", rusqlite::params![name])?;
        Ok(record)
    }

    /// 获取安装记录数量
    #[allow(dead_code)]
    pub fn count(&self) -> Result<i64> {
        let count: i64 = self
            .db
            .query_row("SELECT COUNT(*) FROM installed", [], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_record_json_roundtrip() {
        let record = InstallRecord {
            id: "test-uuid".to_string(),
            name: "test-pkg".to_string(),
            version: Some("1.0.0".to_string()),
            install_path: "/tmp/test".to_string(),
            archive_url: Some("https://example.com/test.zip".to_string()),
            sha256: Some("abc123".to_string()),
            path_entries: vec!["/tmp/test/bin".to_string()],
            config_files: vec!["/tmp/test/config.ini".to_string()],
            installed_at: "2024-01-01T00:00:00Z".to_string(),
            original_path: Some("/usr/bin".to_string()),
            env_vars_backup: serde_json::json!({"PATH": "/usr/bin"}),
        };

        // 序列化
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("test-pkg"));
        assert!(json.contains("1.0.0"));

        // 反序列化
        let deserialized: InstallRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test-pkg");
        assert_eq!(deserialized.version, Some("1.0.0".to_string()));
        assert_eq!(deserialized.path_entries.len(), 1);
    }

    #[test]
    fn test_path_entries_json_roundtrip() {
        let entries = vec!["/path/a".to_string(), "/path/b".to_string()];
        let json = serde_json::to_string(&entries).unwrap();
        let parsed: Vec<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, entries);
    }

    #[test]
    fn test_empty_path_entries_json() {
        let entries: Vec<String> = vec![];
        let json = serde_json::to_string(&entries).unwrap();
        let parsed: Vec<String> = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_manifest_open_and_crud() {
        // 使用临时数据库路径（覆盖 db_dir 环境变量不可行，
        // 所以直接用 open() 测试真实 db 的基本 CRUD，测试后清理）
        let manifest = match Manifest::open() {
            Ok(m) => m,
            Err(_) => return, // 环境不支持 SQLite，跳过
        };

        // 插入一条测试记录
        let record = InstallRecord {
            id: "test-crud-uuid".to_string(),
            name: "test-crud-pkg".to_string(),
            version: Some("2.0.0".to_string()),
            install_path: "/tmp/crud-test".to_string(),
            archive_url: None,
            sha256: None,
            path_entries: vec![],
            config_files: vec![],
            installed_at: "2024-01-01T00:00:00Z".to_string(),
            original_path: None,
            env_vars_backup: serde_json::json!({}),
        };

        // 如果已存在先删除
        let _ = manifest.remove("test-crud-pkg");

        // 添加
        let id = manifest.add(&record).unwrap();
        assert_eq!(id, "test-crud-uuid");

        // 查询
        let got = manifest.get("test-crud-pkg").unwrap();
        assert!(got.is_some());
        assert_eq!(got.unwrap().version, Some("2.0.0".to_string()));

        // 删除
        let removed = manifest.remove("test-crud-pkg").unwrap();
        assert!(removed.is_some());

        // 确认已删除
        let gone = manifest.get("test-crud-pkg").unwrap();
        assert!(gone.is_none());
    }
}
