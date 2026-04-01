use anyhow::anyhow;
use std::fs;
use std::path;
use std::path::Path;
use toolkit_rs::AppResult;
#[allow(unused_variables)]
pub fn set_permissions(filepath: &str) -> AppResult {
    #[cfg(unix)]
    {
        // 默认权限为可读写，可执行
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        let perm = Permissions::from_mode(0o755);
        match fs::set_permissions(filepath, perm) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow!(format!(
                    "set program permission failed:{}",
                    e.to_string()
                )));
            }
        };
    }
    Ok(())
}

pub fn clear_dir(dir_path: &str) -> AppResult {
    for entry in fs::read_dir(dir_path)? {
        let entry_path = entry?.path();
        if entry_path.is_dir() {
            fs::remove_dir_all(&entry_path)?;
        } else {
            fs::remove_file(&entry_path)?;
        }
    }
    Ok(())
}

//修改文件名
pub fn file_rename(filepath: &str, new_name: &str) -> AppResult {
    let fpath = Path::new(filepath);
    if !fpath.exists() {
        return Err(anyhow!("file:{} not exist", filepath));
    }
    if !fpath.is_file() {
        return Err(anyhow!("filepath:{} is not a file ", filepath));
    }

    let new_file_name = fpath.with_file_name(new_name);

    match fs::rename(fpath, new_file_name) {
        Ok(_) => {}
        Err(e) => {
            return Err(anyhow!(format!(
                "rename file_path:{} to new_name:{} failed:{}",
                filepath,
                new_name,
                e.to_string()
            )));
        }
    }

    Ok(())
}

#[derive(Debug)]
pub struct Move<'a> {
    source: &'a path::Path,
    temp: Option<&'a path::Path>,
}
impl<'a> Move<'a> {
    pub fn from_source(source: &'a path::Path) -> Move<'a> {
        Self { source, temp: None }
    }

    pub fn replace_using_temp(&mut self, temp: &'a path::Path) -> &mut Self {
        self.temp = Some(temp);
        self
    }

    pub fn to_dest(&self, dest: &path::Path) -> AppResult {
        match self.temp {
            None => {
                fs::rename(self.source, dest)?;
            }
            Some(temp) => {
                if dest.exists() {
                    fs::rename(dest, temp)?;
                    if let Err(e) = fs::rename(self.source, dest) {
                        fs::rename(temp, dest)?;
                        return Err(anyhow!(e));
                    }
                } else {
                    fs::rename(self.source, dest)?;
                }
            }
        };
        Ok(())
    }
}

#[test]
fn test_move() {
    let source = Path::new("F:\\crate\\upgrade\\app\\temp\\unzip\\app.exe");
    let dest = Path::new("F:\\crate\\upgrade\\app.exe");
    let res = fs::rename(source, dest);
    match res {
        Ok(_) => {
            println!("File renamed successfully");
        }
        Err(e) => {
            println!("Failed to rename file: {}", e);
        }
    }
}

#[test]
fn test_move2() {
    let test_dir = std::env::temp_dir().join(format!(
        "upgrade_test_move_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    fs::create_dir_all(&test_dir).unwrap();

    let source_path = test_dir.join("source.exe");
    let dest_path = test_dir.join("dest.exe");

    fs::write(&source_path, b"new-binary").unwrap();
    fs::write(&dest_path, b"old-binary").unwrap();

    let temp_path = test_dir.join("dest.exe.bak");

    Move::from_source(&source_path)
        .replace_using_temp(&temp_path)
        .to_dest(&dest_path)
        .unwrap();

    assert!(!source_path.exists());
    assert_eq!(fs::read(&dest_path).unwrap(), b"new-binary");
    assert_eq!(fs::read(&temp_path).unwrap(), b"old-binary");

    fs::remove_dir_all(&test_dir).unwrap();
}
