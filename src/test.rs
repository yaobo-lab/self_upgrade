use crate::{Conf, IUpgrade, UpgradeStatus};
use async_trait::async_trait;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use toolkit_rs::logger::{self, LogConfig, LogStyle};
struct BuildRollbackTestUpgrade {
    conf: Conf,
}

#[async_trait]
impl IUpgrade for BuildRollbackTestUpgrade {
    fn get_config(&self) -> &Conf {
        &self.conf
    }
}

fn init_log() {
    let cfg = LogConfig {
        style: LogStyle::Line,
        ..LogConfig::default()
    };
    logger::setup(cfg).unwrap_or_else(|err| {
        eprintln!("log setup err: {err}");
        process::exit(1);
    });
}

fn make_upgrade() -> BuildRollbackTestUpgrade {
    let app_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let temp_path = app_path.join("temp");

    let conf = Conf {
        current_version: "1.0.0".to_string(),
        upgrade_version: "1.0.1".to_string(),
        install_path: app_path.to_string_lossy().into_owned(),
        bin_name: "app.exe".to_string(),
        download_file_url: "".into(),
        download_file_md5: "3327CD4EF4EBF003A1F907032B2E697D".to_string(),
        download_dir: temp_path.join("download").to_string_lossy().into_owned(),
        backup_dir: temp_path.join("backup").to_string_lossy().into_owned(),
        unzip_dir: temp_path.join("unzip").to_string_lossy().into_owned(),
        to_backup_dir: vec![
            app_path.join("app.exe").to_string_lossy().into_owned(),
            app_path.join("etc").to_string_lossy().into_owned(),
        ],
        need_clear_dir: false,
        on_progress: Some(Arc::new(Box::new(|status, progress| match status {
            UpgradeStatus::Download(v) => {
                log::info!(
                    "download progress: {}% upgrade progress: {}",
                    v,
                    progress.unwrap_or(0)
                );
            }
            UpgradeStatus::Unzip => {
                log::info!("unzipping package");
            }
            UpgradeStatus::Backup => {
                log::info!("backing up files");
            }
            UpgradeStatus::Replace => {
                log::info!("replacing executable");
            }
            UpgradeStatus::Success => {
                log::info!("upgrade success");
            }
            UpgradeStatus::RollBack(msg) => {
                log::warn!("rollback: {msg}");
            }
            UpgradeStatus::Failed(msg) => {
                log::info!("upgrade failed: {msg}");
            }
        }))),
        on_roll_back: Some(Arc::new(Box::new(|err| {
            log::info!("rollback: {err}");
        }))),
        ..Conf::default()
    };
    println!("{}", conf.format_all_fields());
    BuildRollbackTestUpgrade { conf }
}

#[tokio::test]
async fn upgrade_rolls_back_with_build_app_and_zip() {
    init_log();
    let upgrade = make_upgrade();
    let result = match upgrade.upgrade().await {
        Ok(v) => v,
        Err(e) => {
            log::error!("upgrade failed: {e}");
            false
        }
    };
    log::info!("Upgrade result: {result}");
    // fs::remove_dir_all(&fixture.root).expect("cleanup test directory");
}
