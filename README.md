# self_upgrade

一个用于独立执行文件自升级的 Rust 库，适合“下载升级包 -> 校验 -> 解压 -> 备份 -> 替换可执行文件 -> 重启新程序 -> 失败回滚”这类场景。

当前仓库主要提供库能力，示例程序位于 `examples/demo.rs`。

## 功能特性

- 基于版本号判断是否需要升级
- 下载升级包并校验 MD5
- 自动解压升级包
- 升级前备份指定目录
- 替换主程序文件
- 启动新进程做升级确认
- 升级失败自动回滚
- 支持进度回调和回滚回调
- 支持复用已下载且 MD5 匹配的升级包

## 目录结构

```text
.
├─src/
│  ├─appbin.rs      # 应用升级实现
│  ├─config.rs      # 升级配置
│  ├─status.rs      # 升级状态
│  ├─types.rs       # 文件与目录辅助能力
│  ├─upgrade.rs     # 核心升级流程与 trait
│  └─version.rs     # 版本比较
├─examples/
│  └─demo.rs        # 接入示例
├─etc/
├─logs/
└─Cargo.toml
```

## 环境要求

- Rust 2021
- 推荐使用 `tokio` 异步运行时
- Windows / Unix 均可使用

> Windows 下如果 `bin_name` 未带 `.exe`，库会自动补全。

## 安装

如果你要在自己的项目中使用，可以在 `Cargo.toml` 中加入：

```toml
[dependencies]
self_upgrade = { path = "." }
```

如果这是另一个仓库，请改成对应的 git 或私有源依赖方式。

## 核心用法

### 1. 引入类型

```rust
use self_upgrade::{AppBinUpgrade, Conf, IUpgrade, UpgradeStatus};
```

### 2. 构造配置

```rust
use std::sync::Arc;
use self_upgrade::{Conf, UpgradeStatus};

let mut cfg = Conf {
    current_version: "1.0.0".to_string(),
    upgrade_version: "1.0.1".to_string(),
    install_path: "./".to_string(),
    bin_name: "app.exe".to_string(),
    download_file_url: "https://example.com/app.zip".to_string(),
    download_file_md5: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string(),
    to_backup_dir: vec!["./configs".to_string()],
    need_clear_dir: true,
    on_progress: Some(Arc::new(Box::new(|status, progress| {
        println!("status: {:?}, progress: {:?}", status, progress);
    }))),
    on_roll_back: Some(Arc::new(Box::new(|msg| {
        eprintln!("rollback: {msg}");
    }))),
    ..Conf::default()
};

cfg.check()?;
```

### 3. 执行升级

```rust
use self_upgrade::{AppBinUpgrade, IUpgrade};

AppBinUpgrade::default()
    .config(cfg)
    .upgrade()
    .await?;
```

## 完整示例

```rust
use std::sync::Arc;
use self_upgrade::{AppBinUpgrade, Conf, IUpgrade, UpgradeStatus};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cfg = Conf {
        current_version: "1.0.0".to_string(),
        upgrade_version: "1.0.1".to_string(),
        install_path: "./".to_string(),
        bin_name: "app.exe".to_string(),
        download_file_url: "https://example.com/app.zip".to_string(),
        download_file_md5: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string(),
        to_backup_dir: vec!["./configs".to_string()],
        need_clear_dir: true,
        on_progress: Some(Arc::new(Box::new(|status, progress| match status {
            UpgradeStatus::Download(p) => println!("download: {p}% / {:?}", progress),
            UpgradeStatus::Unzip => println!("unzip"),
            UpgradeStatus::Backup => println!("backup"),
            UpgradeStatus::Replace => println!("replace"),
            UpgradeStatus::Success => println!("success"),
            UpgradeStatus::RollBack(msg) => println!("rollback: {msg}"),
            UpgradeStatus::Failed(msg) => println!("failed: {msg}"),
        }))),
        on_roll_back: Some(Arc::new(Box::new(|msg| {
            eprintln!("rollback callback: {msg}");
        }))),
        ..Conf::default()
    };

    cfg.check()?;

    let ok = AppBinUpgrade::default().config(cfg).upgrade().await?;
    println!("upgrade result: {ok}");

    Ok(())
}
```

## 配置说明

`Conf` 是升级的核心配置：

| 字段 | 说明 |
| --- | --- |
| `upgrade_version` | 目标版本号 |
| `current_version` | 当前版本号 |
| `install_path` | 程序安装目录 |
| `bin_name` | 主程序文件名 |
| `download_file_url` | 升级包下载地址 |
| `download_file_md5` | 升级包 MD5 |
| `download_dir` | 升级包下载目录，默认 `./tmp/download/` |
| `backup_dir` | 备份目录，默认 `./tmp/backup/` |
| `unzip_dir` | 解压目录，默认 `./tmp/unzip/` |
| `to_backup_dir` | 需要备份的目录列表 |
| `need_clear_dir` | 升级结束后是否清理临时目录 |
| `on_progress` | 升级进度回调 |
| `on_roll_back` | 回滚回调 |

## 升级流程

库内的升级顺序如下：

1. 判断版本是否需要升级
2. 创建下载、备份、解压目录
3. 下载升级包，若本地已有相同 MD5 文件则直接复用
4. 解压升级包
5. 备份 `to_backup_dir` 中指定内容
6. 替换主程序与解压后的其他文件
7. 启动新进程并等待返回 `ok`
8. 清理临时目录
9. 任一步骤失败则执行回滚

## 回调状态说明

`UpgradeStatus` 当前包含以下状态：

- `Download(u8)`：下载中
- `Unzip`：解压中
- `Backup`：备份中
- `Replace`：替换文件中
- `Success`：升级完成
- `RollBack(String)`：回滚中
- `Failed(String)`：升级失败

## 使用约束与注意事项

### 1. 版本号必须符合 semver

版本比较使用 `semver`，建议使用如下格式：

```text
1.0.0
1.2.3
2.0.0-beta.1
```

不建议使用 `1.0.0.0` 这类格式。

### 2. 升级包中必须包含目标程序文件

解压后，库会在 `unzip_dir` 下查找与 `bin_name` 同名的文件；如果不存在，升级会失败。

### 3. 新进程需要输出 `ok`

升级完成后，库会重新启动目标程序，并以 `update` 作为命令行参数启动。当前实现会等待子进程结束，并检查标准输出是否为：

```text
ok
```

如果不是，当前升级流程会判定失败。

### 4. `to_backup_dir` 不能为空

`Conf::check()` 会要求 `to_backup_dir` 必填。如果你的业务确实不需要备份，需要先调整项目实现或传入最小可接受目录。

### 5. 示例程序需要替换成真实参数

`examples/demo.rs` 更适合作为接入模板，运行前建议至少确认以下内容：

- 下载地址可访问
- MD5 正确
- 当前版本与目标版本符合 semver
- `install_path` 与 `bin_name` 指向真实程序
- `to_backup_dir` 指向实际存在的目录

## 运行示例

仅编译示例：

```bash
cargo run --example demo
```

但在实际运行前，请先按上一节修改示例中的配置。

## 开发与测试

编译检查：

```bash
cargo check
```

编译测试：

```bash
cargo test --no-run
```

运行测试：

```bash
cargo test
```

## 已验证信息

当前仓库已通过：

```bash
cargo test --no-run
```

说明项目当前可以完成测试目标的编译。
