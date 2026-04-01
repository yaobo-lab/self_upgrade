use std::process;
use std::sync::Arc;

use self_upgrade::{AppBinUpgrade, Conf, IUpgrade, UpgradeStatus};
use toolkit_rs::logger::{self, LogConfig};
fn init_log() {
    logger::setup(LogConfig::default()).unwrap_or_else(|err| {
        eprintln!("log setup err: {err}");
        process::exit(1);
    });
}

#[tokio::main]
async fn main() {
    init_log();

    let current_exe = std::env::current_exe().unwrap_or_else(|err| {
        eprintln!("failed to locate current executable: {err}");
        process::exit(1);
    });

    let bin_dir = current_exe
        .parent()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string());

    let bin_name = current_exe
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| {
            eprintln!("failed to read current executable name");
            process::exit(1);
        });

    let mut cfg = Conf {
        current_version: "1.0.0.0".to_string(),
        upgrade_version: "1.0.0.1".to_string(),
        install_path: bin_dir,
        bin_name,
        download_file_url:
            "https://xxxx.cn/pack/2024/9/87eaa5bd3df75764d82ee28889bc52d1.zip?e=xxxBG4t1x5X8="
                .to_string(),
        download_file_md5: "87eaa5bd3df75764d82ee28889bc52d1".to_string(),
        to_backup_dir: vec!["./configs/".to_string()],
        need_clear_dir: true,
        on_progress: Some(Arc::new(Box::new(|status, progress| match status {
            UpgradeStatus::Download(_) => {
                log::info!("download progress: {}%", progress.unwrap_or(0));
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
                log::error!("upgrade failed: {msg}");
            }
        }))),
        on_roll_back: Some(Arc::new(Box::new(|err| {
            log::error!("upgrade failed, start rollback: {err}");
        }))),
        ..Conf::default()
    };

    cfg.check().unwrap_or_else(|err| {
        eprintln!("config check failed: {err}");
        process::exit(1);
    });

    AppBinUpgrade::default()
        .config(cfg)
        .upgrade()
        .await
        .unwrap_or_else(|err| {
            eprintln!("upgrade run failed: {err}");
            process::exit(1);
        });
}
