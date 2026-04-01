use crate::{config::*, IUpgrade};

// 应用二进制文件升级
#[derive(Default)]
pub struct AppBinUpgrade {
    pub conf: Conf,
}

impl AppBinUpgrade {
    pub fn config(mut self, c: Conf) -> Self {
        self.conf = c;
        self
    }
}

impl IUpgrade for AppBinUpgrade {
    fn get_config(&self) -> &Conf {
        &self.conf
    }
}
