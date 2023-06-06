use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

pub fn get_images(
    root: &Path,
    output_dir: Option<PathBuf>,
    file_types: Vec<&str>,
    ending: &Option<String>,
) -> Result<Vec<(PathBuf, PathBuf)>, String> {
    let v = get_all_files_with(root, &file_types)?;
    let multiple = v.len() != 1;
    let v = v
        .iter()
        .map(|v| {
            (
                v.path().to_path_buf(),
                generate_output_path(root, v.path(), output_dir.clone(), multiple, ending),
            )
        })
        .collect::<Vec<_>>();
    let mut res = vec![];
    for (input, output) in v {
        let output = output?;
        if output.exists() {
            println!("Skipping {} because it already exists", output.display());
            continue;
        }
        std::fs::create_dir_all(output.parent().unwrap_or(Path::new(".")))
            .map_err(|e| format!("Failed to create dir: {}", e))?;
        res.push((input, output));
    }
    Ok(res)
}

pub fn get_all_files_with(root: &Path, file_types: &[&str]) -> Result<Vec<DirEntry>, String> {
    let temp = WalkDir::new(root)
        .into_iter()
        .map(|v| v.map_err(|e| format!("Failed to get entry: {}", e)))
        .collect::<Vec<_>>();
    let mut res = vec![];
    for item in temp {
        let item = item?;
        if item.path().is_file()
            && file_name_checker(item.file_name().to_str().unwrap_or(""), file_types)
        {
            res.push(item);
        }
    }
    Ok(res)
}

pub fn generate_output_path(
    root: &Path,
    file_dir: &Path,
    output_dir: Option<PathBuf>,
    multiple: bool,
    ending: &Option<String>,
) -> Result<PathBuf, String> {
    let root = if root.is_file() {
        root.parent().unwrap_or(Path::new("."))
    } else {
        root
    };
    let output_dir = output_dir.unwrap_or(generate_output(root));
    let relative_path = file_dir
        .strip_prefix(root)
        .map_err(|e| format!("Failed to strip prefix: {}", e))?;
    let mut res = if multiple {
        output_dir.join(relative_path)
    } else {
        let file = output_dir.file_name();
        let mut res = output_dir.clone();
        if let Some(filen) = file {
            if !filen.to_str().unwrap_or("").contains('.') {
                res = res.join(relative_path);
                let filename = format!(
                    "{}-output",
                    res.file_name()
                        .map(|v| v.to_str().unwrap_or(""))
                        .unwrap_or("")
                );
                res.set_file_name(filename);
            }
        }
        res
    };
    if let Some(ending) = ending {
        res.set_extension(ending);
    }
    Ok(res)
}

fn generate_output(path: &Path) -> PathBuf {
    let mut v = path
        .iter()
        .map(|v| v.to_str().unwrap_or("").to_string())
        .collect::<Vec<_>>();
    let last = format!("{}-output", v.pop().unwrap_or("unknown_file".to_string()));
    v.push(last);
    PathBuf::from_iter(v)
}

fn file_name_checker(name: &str, file_types: &[&str]) -> bool {
    if !name.contains('.') {
        return false;
    }
    if file_types.contains(
        &name
            .split('.')
            .last()
            .expect("checked before")
            .to_lowercase()
            .as_str(),
    ) {
        return true;
    }
    false
}
