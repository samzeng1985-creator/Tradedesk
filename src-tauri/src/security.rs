use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;
use zeroize::Zeroizing;

const BACKUP_MAGIC: &[u8] = b"TRADEDESK-BACKUP-1\n";

fn io_error(context: &str, error: std::io::Error) -> String {
    format!("{context}：{error}")
}

fn database_error(context: &str, error: rusqlite::Error) -> String {
    format!("{context}：{error}")
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".tmp-{}", Uuid::new_v4()));
    path.with_file_name(name)
}

fn safety_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".before-restore");
    path.with_file_name(name)
}

fn sqlite_sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

pub fn recovery_vault_exists(path: &Path) -> bool {
    path.is_file()
}

pub fn restore_pending(database_path: &Path) -> bool {
    safety_path(database_path).is_file()
}

pub fn commit_restored_workspace(database_path: &Path, recovery_path: &Path) -> Result<(), String> {
    for path in [safety_path(database_path), safety_path(recovery_path)] {
        if path.exists() {
            fs::remove_file(path).map_err(|error| io_error("无法清理已验证的恢复副本", error))?;
        }
    }
    Ok(())
}

pub fn rollback_restored_workspace(
    database_path: &Path,
    recovery_path: &Path,
) -> Result<(), String> {
    let database_safety = safety_path(database_path);
    let recovery_safety = safety_path(recovery_path);
    if !database_safety.exists() {
        return Err("没有可以撤销的备份恢复。".to_owned());
    }
    if database_path.exists() {
        fs::remove_file(database_path).map_err(|error| io_error("无法移除待验证数据库", error))?;
    }
    if recovery_path.exists() {
        fs::remove_file(recovery_path)
            .map_err(|error| io_error("无法移除待验证恢复密钥库", error))?;
    }
    fs::rename(&database_safety, database_path)
        .map_err(|error| io_error("无法恢复原数据库", error))?;
    if recovery_safety.exists() {
        fs::rename(&recovery_safety, recovery_path)
            .map_err(|error| io_error("无法恢复原密钥库", error))?;
    }
    for path in [
        sqlite_sidecar(database_path, "-wal"),
        sqlite_sidecar(database_path, "-shm"),
    ] {
        if path.exists() {
            fs::remove_file(path).map_err(|error| io_error("无法完成撤销恢复清理", error))?;
        }
    }
    Ok(())
}

pub fn create_recovery_vault(path: &Path, password: &str) -> Result<String, String> {
    let recovery_key = format!("TDK-{}-{}", Uuid::new_v4(), Uuid::new_v4());
    let temporary = temporary_path(path);
    let connection = Connection::open(&temporary)
        .map_err(|error| database_error("无法创建恢复密钥库", error))?;
    connection
        .pragma_update(None, "key", &recovery_key)
        .map_err(|error| database_error("无法加密恢复密钥库", error))?;
    #[cfg(not(target_os = "windows"))]
    connection
        .pragma_update(None, "cipher_memory_security", "ON")
        .map_err(|error| database_error("无法启用密钥内存保护", error))?;
    connection
        .execute_batch(
            "PRAGMA journal_mode = DELETE;
             PRAGMA synchronous = FULL;
             CREATE TABLE recovery_secret(
                id INTEGER PRIMARY KEY CHECK(id = 1),
                workspace_password TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
             );",
        )
        .map_err(|error| database_error("无法初始化恢复密钥库", error))?;
    connection
        .execute(
            "INSERT INTO recovery_secret(id, workspace_password) VALUES(1, ?1)",
            params![password],
        )
        .map_err(|error| database_error("无法写入恢复信息", error))?;
    connection
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|error| database_error("无法完成恢复密钥库写入", error))?;
    drop(connection);

    let previous = safety_path(path);
    if previous.exists() {
        fs::remove_file(&previous).map_err(|error| io_error("无法清理旧恢复密钥副本", error))?;
    }
    if path.exists() {
        fs::rename(path, &previous).map_err(|error| io_error("无法保留旧恢复密钥库", error))?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if previous.exists() {
            let _ = fs::rename(&previous, path);
        }
        return Err(io_error("无法启用新恢复密钥", error));
    }
    if previous.exists() {
        fs::remove_file(previous).map_err(|error| io_error("无法清理旧恢复密钥库", error))?;
    }
    Ok(recovery_key)
}

pub fn recover_password(
    path: &Path,
    recovery_key: Zeroizing<String>,
) -> Result<Zeroizing<String>, String> {
    if !path.is_file() {
        return Err("当前工作区还没有恢复密钥，请使用原密码解锁后生成。".to_owned());
    }
    let connection =
        Connection::open(path).map_err(|error| database_error("无法打开恢复密钥库", error))?;
    connection
        .pragma_update(None, "key", recovery_key.as_str())
        .map_err(|error| database_error("恢复密钥无效", error))?;
    let password = connection
        .query_row(
            "SELECT workspace_password FROM recovery_secret WHERE id = 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|_| "恢复密钥不正确或恢复信息已经损坏。".to_owned())?
        .ok_or_else(|| "恢复密钥库中没有有效的工作区信息。".to_owned())?;
    Ok(Zeroizing::new(password))
}

pub fn create_backup_package(
    database_backup: &Path,
    recovery_vault: &Path,
    output: &Path,
) -> Result<u64, String> {
    let recovery =
        fs::read(recovery_vault).map_err(|error| io_error("无法读取恢复密钥库", error))?;
    let mut database =
        fs::File::open(database_backup).map_err(|error| io_error("无法读取数据库备份", error))?;
    let temporary = temporary_path(output);
    let mut package =
        fs::File::create(&temporary).map_err(|error| io_error("无法创建备份包", error))?;
    package
        .write_all(BACKUP_MAGIC)
        .map_err(|error| io_error("无法写入备份头", error))?;
    package
        .write_all(&(recovery.len() as u64).to_le_bytes())
        .map_err(|error| io_error("无法写入备份索引", error))?;
    package
        .write_all(&recovery)
        .map_err(|error| io_error("无法写入恢复信息", error))?;
    std::io::copy(&mut database, &mut package)
        .map_err(|error| io_error("无法写入数据库备份", error))?;
    package
        .sync_all()
        .map_err(|error| io_error("无法将备份同步到磁盘", error))?;
    drop(package);
    fs::rename(&temporary, output).map_err(|error| io_error("无法完成备份文件", error))?;
    fs::metadata(output)
        .map(|metadata| metadata.len())
        .map_err(|error| io_error("无法读取备份文件信息", error))
}

pub fn restore_backup_package(
    bytes: &[u8],
    database_path: &Path,
    recovery_path: &Path,
) -> Result<(), String> {
    if !bytes.starts_with(BACKUP_MAGIC) || bytes.len() < BACKUP_MAGIC.len() + 8 + 256 {
        return Err("所选文件不是有效的 TradeDesk 0.14 备份包。".to_owned());
    }
    let length_offset = BACKUP_MAGIC.len();
    let recovery_length = u64::from_le_bytes(
        bytes[length_offset..length_offset + 8]
            .try_into()
            .map_err(|_| "备份索引已损坏。".to_owned())?,
    ) as usize;
    let recovery_start = length_offset + 8;
    let database_start = recovery_start
        .checked_add(recovery_length)
        .ok_or_else(|| "备份索引超出有效范围。".to_owned())?;
    if recovery_length < 128 || database_start >= bytes.len() || bytes.len() - database_start < 128
    {
        return Err("备份包内容不完整或已经损坏。".to_owned());
    }

    if let Some(parent) = database_path.parent() {
        fs::create_dir_all(parent).map_err(|error| io_error("无法创建工作区目录", error))?;
    }
    let database_temp = temporary_path(database_path);
    let recovery_temp = temporary_path(recovery_path);
    fs::write(&database_temp, &bytes[database_start..])
        .map_err(|error| io_error("无法写入恢复数据库", error))?;
    fs::write(&recovery_temp, &bytes[recovery_start..database_start])
        .map_err(|error| io_error("无法写入恢复密钥库", error))?;

    let database_safety = safety_path(database_path);
    let recovery_safety = safety_path(recovery_path);
    if database_safety.exists() || recovery_safety.exists() {
        return Err("上一次备份恢复尚未验证，请先成功解锁或撤销上次恢复。".to_owned());
    }
    if database_path.exists() {
        fs::rename(database_path, &database_safety)
            .map_err(|error| io_error("无法保留当前数据库", error))?;
    }
    if recovery_path.exists()
        && let Err(error) = fs::rename(recovery_path, &recovery_safety)
    {
        if database_safety.exists() {
            let _ = fs::rename(&database_safety, database_path);
        }
        return Err(io_error("无法保留当前恢复密钥库", error));
    }

    let install_result = fs::rename(&database_temp, database_path)
        .and_then(|_| fs::rename(&recovery_temp, recovery_path));
    if let Err(error) = install_result {
        let _ = fs::remove_file(database_path);
        let _ = fs::remove_file(recovery_path);
        if database_safety.exists() {
            let _ = fs::rename(&database_safety, database_path);
        }
        if recovery_safety.exists() {
            let _ = fs::rename(&recovery_safety, recovery_path);
        }
        return Err(io_error("无法安装备份数据", error));
    }

    for path in [
        sqlite_sidecar(database_path, "-wal"),
        sqlite_sidecar(database_path, "-shm"),
    ] {
        if path.exists() {
            fs::remove_file(path).map_err(|error| io_error("无法完成恢复清理", error))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_vault_encrypts_and_recovers_workspace_password() {
        let root = std::env::temp_dir().join(format!("tradedesk-security-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let vault = root.join("workspace.recovery.tdesk");
        let workspace_password = "correct-horse-battery-staple";

        let recovery_key = create_recovery_vault(&vault, workspace_password).unwrap();
        let raw = fs::read(&vault).unwrap();
        assert!(
            !raw.windows(workspace_password.len())
                .any(|window| window == workspace_password.as_bytes())
        );
        assert_eq!(
            recover_password(&vault, Zeroizing::new(recovery_key))
                .unwrap()
                .as_str(),
            workspace_password
        );
        assert!(recover_password(&vault, Zeroizing::new("TDK-wrong-key".to_owned())).is_err());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn backup_package_round_trips_database_and_recovery_files() {
        let root = std::env::temp_dir().join(format!("tradedesk-backup-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let source_database = root.join("source.tdesk");
        let source_recovery = root.join("source.recovery.tdesk");
        let package = root.join("backup.tdbackup");
        let restored_database = root.join("restored.tdesk");
        let restored_recovery = root.join("restored.recovery.tdesk");
        let database_bytes = vec![42_u8; 512];
        let recovery_bytes = vec![77_u8; 384];
        let original_database_bytes = vec![11_u8; 512];
        let original_recovery_bytes = vec![22_u8; 384];
        fs::write(&source_database, &database_bytes).unwrap();
        fs::write(&source_recovery, &recovery_bytes).unwrap();
        fs::write(&restored_database, &original_database_bytes).unwrap();
        fs::write(&restored_recovery, &original_recovery_bytes).unwrap();

        create_backup_package(&source_database, &source_recovery, &package).unwrap();
        let package_bytes = fs::read(&package).unwrap();
        restore_backup_package(&package_bytes, &restored_database, &restored_recovery).unwrap();

        assert_eq!(fs::read(&restored_database).unwrap(), database_bytes);
        assert_eq!(fs::read(&restored_recovery).unwrap(), recovery_bytes);
        assert!(restore_pending(&restored_database));
        rollback_restored_workspace(&restored_database, &restored_recovery).unwrap();
        assert_eq!(
            fs::read(&restored_database).unwrap(),
            original_database_bytes
        );
        assert_eq!(
            fs::read(&restored_recovery).unwrap(),
            original_recovery_bytes
        );
        assert!(!restore_pending(&restored_database));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_backup_packages_never_replace_the_current_workspace() {
        let root =
            std::env::temp_dir().join(format!("tradedesk-invalid-backup-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let database = root.join("workspace.tdesk");
        let recovery = root.join("workspace.recovery.tdesk");
        let original_database = vec![31_u8; 512];
        let original_recovery = vec![47_u8; 384];
        fs::write(&database, &original_database).unwrap();
        fs::write(&recovery, &original_recovery).unwrap();

        for invalid in [vec![0_u8; 512], BACKUP_MAGIC.to_vec()] {
            assert!(restore_backup_package(&invalid, &database, &recovery).is_err());
            assert_eq!(fs::read(&database).unwrap(), original_database);
            assert_eq!(fs::read(&recovery).unwrap(), original_recovery);
            assert!(!restore_pending(&database));
        }

        fs::remove_dir_all(root).unwrap();
    }
}
