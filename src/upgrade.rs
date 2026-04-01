use crate::{version, Conf, Move, UpgradeStatus};
use anyhow::anyhow;
use async_trait::async_trait;
use downloader::{get_file_md5, HttpDownload};
use fs_extra_rs::{self, dir::CopyOptions};
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use std::{fs, u8};
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::time::sleep;
use toolkit_rs::AppResult;

#[async_trait]
pub trait IUpgrade {
    fn get_config(&self) -> &Conf;

    fn check_need_upgrade(&self) -> bool {
        let cfg = self.get_config();
        version::is_greater(&cfg.current_version, &cfg.upgrade_version).unwrap_or(false)
    }

    async fn clear_tmp_dir(&self) -> AppResult {
        let cfg = self.get_config();
        if !cfg.need_clear_dir {
            return Ok(());
        }

        log::info!("[upgrade] [6].clear tmp dir");

        let backup_dir = cfg.backup_dir.clone();
        crate::clear_dir(&backup_dir)
            .map_err(|e| anyhow!(format!("clear dir:{} err:{}", backup_dir, e)))?;

        let unzip_dir = cfg.unzip_dir.clone();
        crate::clear_dir(&unzip_dir)
            .map_err(|e| anyhow!(format!("clear dir:{} err:{}", unzip_dir, e)))?;

        let progress = cfg.on_progress.clone();
        if let Some(f) = progress.clone() {
            f(UpgradeStatus::Success, Some(100));
        }
        log::info!("[upgrade] [6].clear tmp dir success");
        Ok(())
    }

    fn create_dir(&self) -> AppResult {
        let cfg = self.get_config();

        fs_extra_rs::dir::create_all(cfg.backup_dir.as_str(), false)
            .map_err(|e| anyhow!(format!("create {} dir failed:{}", cfg.backup_dir, e)))?;

        fs_extra_rs::dir::create_all(cfg.download_dir.as_str(), false)
            .map_err(|e| anyhow!(format!("create {} dir failed:{}", cfg.download_dir, e)))?;

        fs_extra_rs::dir::create_all(cfg.unzip_dir.as_str(), false)
            .map_err(|e| anyhow!(format!("create {} dir failed:{}", cfg.unzip_dir, e)))?;

        Ok(())
    }

    fn find_downloaded_file_by_md5(&self) -> AppResult<Option<PathBuf>> {
        let cfg = self.get_config();
        let download_dir = Path::new(&cfg.download_dir);
        if !download_dir.exists() {
            return Ok(None);
        }

        for entry in fs::read_dir(download_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_md5 = match get_file_md5(path.to_string_lossy().as_ref()) {
                Ok(md5) => md5,
                Err(err) => {
                    log::warn!(
                        "[upgrade] skip cached file md5 check failed, path:{}, err:{}",
                        path.display(),
                        err
                    );
                    continue;
                }
            };

            if file_md5.eq_ignore_ascii_case(&cfg.download_file_md5) {
                return Ok(Some(path));
            }
        }

        Ok(None)
    }

    async fn download(&self) -> AppResult<String> {
        //检查是否有已经下载好的文件，避免重复下载
        if let Some(existing_file) = self.find_downloaded_file_by_md5()? {
            log::info!(
                "[upgrade] [1].reuse downloaded file:{}",
                existing_file.display()
            );
            if let Some(f) = self.get_config().on_progress.clone() {
                f(UpgradeStatus::Download(100), Some(40));
            }
            return Ok(existing_file.to_string_lossy().into_owned());
        }

        if let Some(f) = self.get_config().on_progress.clone() {
            f(UpgradeStatus::Download(0), Some(0));
        }

        let cfg = self.get_config();
        let download_dir = &cfg.download_dir;
        let md5 = &cfg.download_file_md5;
        let url = &cfg.download_file_url;

        log::info!("[upgrade] [1].download file md5:{} start", md5);

        let (sender, receiver) = kanal::bounded_async(10);
        let progress = self.get_config().on_progress.clone();
        tokio::spawn(async move {
            while let Ok(p) = receiver.recv().await {
                let upgrade_progress = 10 + (p / 100 * 30);
                if let Some(ref f) = progress {
                    let f = f.clone();
                    f(UpgradeStatus::Download(p), Some(upgrade_progress as i32));
                }
            }
            log::debug!("[upgrade] download process end");
        });

        let callback = Box::new(move |p: u8| {
            let _ = sender.try_send(p);
        });

        let zipfile = HttpDownload::new()
            .set_max_retries(3)
            .set_num_workers(1)
            .debug(false)
            .on_down_progress(callback)
            .set_file_md5(md5.to_owned())
            .set_save_dir(download_dir.to_owned())
            .set_url(url.as_str())
            .await
            .map_err(|e| anyhow!(format!("download file:{} failed:{}", url, e)))?
            .start()
            .await
            .map_err(|e| anyhow!(format!("download file:{} failed:{}", url, e)))?;

        log::info!("[upgrade] [1].download file success:{}", zipfile);
        Ok(zipfile)
    }

    fn unzip(&self, zipfile: &str) -> AppResult<PathBuf> {
        let cfg = self.get_config();
        //配置进度
        let progress = cfg.on_progress.clone();
        if let Some(f) = progress.clone() {
            f(UpgradeStatus::Unzip, Some(50));
        }

        let unzip_path = &cfg.unzip_dir;
        log::info!("[upgrade] [2].unzip file:{} start", zipfile);
        downloader::uzip_file(zipfile, unzip_path)?;
        let new_exe = PathBuf::from(unzip_path).join(&cfg.bin_name);
        if !new_exe.exists() {
            return Err(anyhow!("program file:{} not exist", new_exe.display()));
        }
        log::info!("[upgrade] [2].unzip path:{} success", unzip_path);
        Ok(new_exe)
    }

    async fn start_new_process(&self) -> AppResult {
        let cfg = self.get_config();

        let app_path = cfg.get_install_app_path();
        let progress = cfg.on_progress.clone();
        let path = Path::new(app_path.as_str());

        log::info!("[upgrade] [5].start new process path:{}", app_path);

        let mut child: Child = match Command::new(path)
            .args(["update"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                return Err(anyhow!(format!(
                    "start new process path:{} failed:{}",
                    app_path, e
                )));
            }
        };

        let result = child.wait().await;
        if let Err(err) = result {
            return Err(anyhow!(format!("failed to wait on child:{}", err)));
        }

        let stdout_result = child.stdout.take();
        let Some(mut stdout) = stdout_result else {
            return Err(anyhow!("start new process println test: is none"));
        };

        let mut buffer = Vec::new();
        stdout.read_to_end(&mut buffer).await?;

        let stdout_str = String::from_utf8_lossy(&buffer).to_string();
        log::info!(
            "[upgrade] start new process println stdout_str:{}",
            stdout_str
        );

        let is_ok = stdout_str.trim() == "ok";
        if !is_ok {
            return Err(anyhow!("start new process failed"));
        }

        if let Some(f) = progress.clone() {
            f(UpgradeStatus::Replace, Some(90));
        }
        log::info!("[upgrade] [5].start new exe:{} success", app_path);
        Ok(())
    }

    async fn backup(&self) -> AppResult {
        let cfg = self.get_config();
        if cfg.to_backup_dir.is_empty() {
            return Ok(());
        }

        let progress = cfg.on_progress.clone();
        if let Some(f) = progress.clone() {
            f(UpgradeStatus::Backup, Some(60));
        }

        let backup_dir = &cfg.backup_dir;

        log::info!("[upgrade] [3].backup to path:{} start", backup_dir);
        crate::clear_dir(backup_dir)
            .map_err(|e| anyhow!(format!("clear backup dir:{} failed:{}", backup_dir, e)))?;

        let mut from_paths = Vec::new();

        for dir in &cfg.to_backup_dir {
            let p = Path::new(dir);
            if !p.exists() {
                log::warn!("[upgrade] backup path:{} not exist, skip", dir);
                continue;
            }
            from_paths.push(format!("{}", p.display()));
        }

        log::info!(
            "[upgrade] [3].to backup files: {:?}, backup dir path:{}",
            from_paths,
            backup_dir
        );

        let options = CopyOptions::new().overwrite(true).skip_exist(false);
        match fs_extra_rs::copy_items(&from_paths, backup_dir, &options) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow!(format!("backup copy file failed :{}", e)));
            }
        }

        log::info!("[upgrade] [3].backup path:{} success", backup_dir);
        Ok(())
    }

    async fn replace_new_exe(&self, new_exe_file_path: &Path) -> AppResult {
        let cfg = self.get_config();

        let app_file_apth = cfg.get_install_app_path();
        log::info!("[upgrade] [4].replace new exe to :{}", app_file_apth);

        let progress = cfg.on_progress.clone();
        if let Some(f) = progress.clone() {
            f(UpgradeStatus::Replace, Some(80));
        }

        if app_file_apth == std::env::current_exe()? {
            self_replace::self_replace(new_exe_file_path)
                .map_err(|e| anyhow!("[upgrade] [4].replace err: self_replace failed: {}", e))?;
        } else {
            Move::from_source(new_exe_file_path.as_ref())
                .to_dest(app_file_apth.as_ref())
                .map_err(|e| {
                    anyhow!(
                        "[upgrade] [4].replace err move failed form_source {}, to_dest: {} err:{}",
                        new_exe_file_path.display(),
                        app_file_apth,
                        e
                    )
                })?;
        }

        //将解压文件夹内容移动到目标目录
        let dirs = fs::read_dir(&cfg.unzip_dir)?;
        let mut from_paths = Vec::new();
        for entry in dirs {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() || path.is_file() {
                from_paths.push(format!("{}", path.display()));
            }
        }
        let options = CopyOptions::new().overwrite(true).skip_exist(false);
        fs_extra_rs::move_items(&from_paths, &cfg.install_path, &options)?;
        log::info!(
            "[upgrade] [4].replace new exe to :{} success",
            cfg.install_path
        );
        Ok(())
    }

    fn roll_back(&self) -> AppResult {
        log::info!("[upgrade] roll back start...");
        let cfg = self.get_config();

        if let Some(f) = cfg.on_roll_back.clone() {
            f("rollback start".to_string());
        }

        let install_path = &cfg.install_path;
        let backup_dir = &cfg.backup_dir;

        let mut from_paths = Vec::new();
        let dirs = fs::read_dir(backup_dir)?;

        for entry in dirs {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() || path.is_file() {
                let r = format!("{}", path.display());
                println!("p----->{}", r);
                from_paths.push(r);
            }
        }

        if let Some(f) = cfg.on_roll_back.clone() {
            f("rollback files".to_string());
        }

        let options = CopyOptions::new().overwrite(true).skip_exist(false);
        match fs_extra_rs::move_items(&from_paths, install_path, &options) {
            Ok(_) => {}
            Err(e) => {
                if let Some(f) = cfg.on_roll_back.clone() {
                    f(format!("failed -> {}", e));
                }
            }
        }

        if let Some(f) = cfg.on_roll_back.clone() {
            f("rollback completed".to_string());
        }
        log::info!("[upgrade] roll back success...");
        Ok(())
    }

    async fn upgrade(&self) -> AppResult<bool> {
        log::info!("[upgrade] start..");
        let cfg = self.get_config();
        log::info!("  * Current Version: {:?}", cfg.current_version);
        log::info!("  * Target Version: {:?}", cfg.upgrade_version);
        log::info!("  * Current App Bin Name: {:?}", cfg.bin_name);
        log::info!("  * Current Install Path: {}", cfg.install_path);
        log::info!("  * Current App Bin Name: {:?}", cfg.bin_name);
        log::info!("  * New exe download url: {:?}", cfg.download_file_url);

        if !self.check_need_upgrade() {
            log::info!("[upgrade] no need upgrade...");
            return Ok(true);
        }
        self.create_dir()?;

        let zipfile = self.download().await?;
        sleep(Duration::from_millis(50)).await;

        if let Err(e) = self._upgrade(zipfile).await {
            log::error!("[upgrade] upgrade failed:{}", e);

            if let Some(f) = cfg.on_progress.clone() {
                f(UpgradeStatus::Failed(e.to_string()), None);
            }

            if let Err(e) = self.roll_back() {
                log::error!("[upgrade] roll back failed:{}", e);
            }

            if let Err(e) = self.clear_tmp_dir().await {
                log::error!("[upgrade] clear tmp dir failed:{}", e);
            }

            return Ok(false);
        }

        Ok(true)
    }

    async fn _upgrade(&self, zipfile: String) -> AppResult {
        //2. ZIP解压
        let new_exe = self.unzip(&zipfile)?;

        //3. 备份旧程序
        self.backup().await?;

        //4. 替换新程序
        sleep(Duration::from_millis(50)).await;
        self.replace_new_exe(&new_exe).await?;

        //5. 启动新程序
        sleep(Duration::from_millis(50)).await;
        self.start_new_process()
            .await
            .map_err(|e| anyhow!("[upgrade] start new process failed:{}", e))?;

        //6. 清理临时目录
        sleep(Duration::from_millis(10)).await;
        if let Err(e) = self.clear_tmp_dir().await {
            log::error!("[upgrade] clear tmp dir failed:{}", e);
        }

        log::info!("[upgrade] successfully..waiting new process to restart...");
        Ok(())
    }
}
