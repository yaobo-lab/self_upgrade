use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeStatus {
    Download(u8),
    Backup,
    Replace,
    Unzip,
    Success,
    RollBack(String),
    Failed(String),
}

impl UpgradeStatus {
    pub fn to_string(&self) -> String {
        match self {
            UpgradeStatus::Download(p) => {
                if *p == 100 {
                    return "下载升级包完成...".to_string();
                }
                format!("下载升级包({}%)...", p)
            }
            UpgradeStatus::Backup => {
                format!("备份程序...")
            }
            UpgradeStatus::Replace => {
                format!("替换旧程序...")
            }
            UpgradeStatus::Unzip => {
                format!("解压升级文件...")
            }
            UpgradeStatus::Success => {
                format!("升级完成")
            }
            UpgradeStatus::RollBack(msg) => {
                format!("回滚操作: {}", msg)
            }
            UpgradeStatus::Failed(msg) => {
                format!("升级失败: {}", msg)
            }
        }
    }
}
