use crate::{ProgressCallback, RollBackCallback};
use anyhow::anyhow;
use std::{path::Path, sync::Arc};
use toolkit_rs::AppResult;

#[derive(Default)]
pub struct Conf {
    // 要更新的版本号
    pub upgrade_version: String,
    // 当前的版本号
    pub current_version: String,
    // 程序安装目录
    pub install_path: String,
    // 可执行程序文件名称
    pub bin_name: String,
    //升级文件下载URL
    pub download_file_url: String,
    //下载文件md5值
    pub download_file_md5: String,
    //下载目录
    pub download_dir: String,
    //备份文件存储目录
    pub backup_dir: String,
    //解压目录
    pub unzip_dir: String,
    // 要备份配置文件目录
    pub to_backup_dir: Vec<String>,
    //是否清理临时目录
    pub need_clear_dir: bool,
    // 升级进度回调
    pub on_progress: Option<Arc<ProgressCallback>>,
    // 回退回调
    pub on_roll_back: Option<Arc<RollBackCallback>>,
}

impl Conf {
    pub fn new() -> Self {
        Conf {
            download_dir: "./tmp/download/".to_string(),
            backup_dir: "./tmp/backup/".to_string(),
            unzip_dir: "./tmp/unzip/".to_string(),
            need_clear_dir: true,
            ..Default::default()
        }
    }
    pub fn set_install_path(mut self, path: String) -> Self {
        self.install_path = path;
        self
    }
    pub fn set_bin_name(mut self, name: String) -> Self {
        self.bin_name = name;
        self
    }

    pub fn set_download_file_url(mut self, url: String) -> Self {
        self.download_file_url = url;
        self
    }
    pub fn set_download_file_md5(mut self, md5: String) -> Self {
        self.download_file_md5 = md5;
        self
    }
    pub fn set_upgrade_version(mut self, version: String) -> Self {
        self.upgrade_version = version;
        self
    }
    pub fn set_current_version(mut self, version: String) -> Self {
        self.current_version = version;
        self
    }
    pub fn set_download_dir(mut self, dir: String) -> Self {
        self.download_dir = dir;
        self
    }
    pub fn set_backup_dir(mut self, dir: String) -> Self {
        self.backup_dir = dir;
        self
    }
    pub fn set_unzip_dir(mut self, dir: String) -> Self {
        self.unzip_dir = dir;
        self
    }
    pub fn set_to_backup_dir(mut self, dirs: Vec<String>) -> Self {
        self.to_backup_dir = dirs;
        self
    }
    pub fn set_on_progress(mut self, callback: ProgressCallback) -> Self {
        self.on_progress = Some(Arc::new(callback));
        self
    }
    pub fn set_on_roll_back(mut self, callback: RollBackCallback) -> Self {
        self.on_roll_back = Some(Arc::new(callback));
        self
    }
    pub fn print_all_fields(&self) {
        log::info!("{}", self.format_all_fields());
    }

    pub fn get_install_app_path(&self) -> String {
        Path::new(&self.install_path)
            .join(&self.bin_name)
            .to_string_lossy()
            .into_owned()
    }

    pub fn format_all_fields(&self) -> String {
        format!(
            concat!(
                "Conf {{\n",
                "  upgrade_version: {:?}\n",
                "  current_version: {:?}\n",
                "  install_path: {:?}\n",
                "  bin_name: {:?}\n",
                "  download_file_url: {:?}\n",
                "  download_file_md5: {:?}\n",
                "  download_dir: {:?}\n",
                "  backup_dir: {:?}\n",
                "  unzip_dir: {:?}\n",
                "  to_backup_dir: {:?}\n",
                "  need_clear_dir: {:?}\n",
                "  on_progress: {}\n",
                "  on_roll_back: {}\n",
                "}}"
            ),
            self.upgrade_version,
            self.current_version,
            self.install_path,
            self.bin_name,
            self.download_file_url,
            self.download_file_md5,
            self.download_dir,
            self.backup_dir,
            self.unzip_dir,
            self.to_backup_dir,
            self.need_clear_dir,
            if self.on_progress.is_some() {
                "Some(<callback>)"
            } else {
                "None"
            },
            if self.on_roll_back.is_some() {
                "Some(<callback>)"
            } else {
                "None"
            }
        )
    }

    pub fn check(&mut self) -> AppResult {
        let mut current_dir = "".to_string();
        if self.install_path.is_empty() {
            match std::env::current_dir() {
                Ok(path) => {
                    let path = path.as_path().display();
                    current_dir = format!("{}", path);
                }
                Err(e) => {
                    return Err(anyhow!("[upgrade] get current dir from env error:{}", e));
                }
            }
            self.install_path = current_dir;
        }

        log::info!("[upgrade] current exe dir:{}", self.install_path);

        if self.upgrade_version.is_empty() {
            return Err(anyhow!("[upgrade] upgrade_version is empty"));
        }
        if self.current_version.is_empty() {
            return Err(anyhow!("[upgrade] current_version is empty"));
        }
        if self.to_backup_dir.is_empty() {
            return Err(anyhow!("[upgrade] to_backup_dir is empty"));
        }
        if self.install_path.is_empty() {
            return Err(anyhow!("[upgrade] execute_dir is empty"));
        }
        if self.bin_name.is_empty() {
            return Err(anyhow!("[upgrade] execute_file_name is empty"));
        }

        if self.download_file_url.is_empty() {
            return Err(anyhow!("[upgrade] download_url is empty"));
        }
        if self.download_file_md5.is_empty() {
            return Err(anyhow!("[upgrade] download_file_md5 is empty"));
        }
        if cfg!(windows) && !self.bin_name.ends_with(".exe") {
            self.bin_name = format!("{}.exe", self.bin_name);
        }

        let path = Path::new(self.install_path.as_str());
        let path = path.join(self.bin_name.as_str());
        if !path.exists() {
            return Err(anyhow!(
                "[upgrade] exe file:{} not exists",
                path.as_path().display()
            ));
        }
        Ok(())
    }
}
