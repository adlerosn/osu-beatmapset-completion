use std::path::PathBuf;

pub fn find_oszs(path: &PathBuf) -> Vec<PathBuf> {
    let mut ret = vec![];
    if path.is_file() || !path.exists() {
        if let Some(ext) = path
            .extension()
            .and_then(|ext| Some(ext.to_str().unwrap().to_string()))
        {
            if ext == "osz" {
                ret.push(path.clone());
            } else if path.exists() {
                if let Some(file) = std::fs::File::open(&path).ok() {
                    if let Some(subfiles) = compress_tools::list_archive_files(file).ok() {
                        ret.append(
                            &mut subfiles
                                .into_iter()
                                .map(|x: String| {
                                    path.join(
                                        x.replace('\\', "/")
                                            .replace('/', &std::path::MAIN_SEPARATOR.to_string()),
                                    )
                                })
                                .filter(|x| {
                                    x.extension().and_then(|ext| ext.to_str()).unwrap_or("")
                                        == "osz"
                                })
                                .collect(),
                        );
                    }
                }
            }
        }
    } else if path.is_dir() {
        for dir_entry in path.read_dir().unwrap() {
            ret.append(&mut find_oszs(&dir_entry.unwrap().path()));
        }
    }
    ret.into_iter()
        .filter(|x| {
            x.file_stem()
                .and_then(|osstr| osstr.to_str())
                .and_then(|stem| {
                    Some(
                        stem.chars()
                            .take_while(|c| c.is_numeric())
                            .collect::<String>(),
                    )
                })
                .and_then(|num| num.parse::<u64>().ok())
                .is_some()
        })
        .collect()
}
