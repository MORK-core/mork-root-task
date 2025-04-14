use std::env;
use std::fs;
use std::path::{Path, PathBuf};

struct MetaInfo {
    pub test_name: String,
    pub file_size: usize,
}

fn main() {
    // 获取环境变量 ROOT_TASK_BIN
    let tests_dir = env::var("TESTS_DIR")
        .expect("TESTS_DIR environment variable not set");
    let path_dir = Path::new(&tests_dir);
    let mut content = String::from("");
    let mut meta_infos = Vec::new();
    for entry in path_dir.read_dir().unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            let file_name = path.file_stem()  // 获取不带后缀的文件名（如 "test"）
                .and_then(|s| s.to_str())    // 转为 &str
                .unwrap_or_default();        // 默认值（避免 panic）
            let data = fs::read(&path).unwrap();
            content.push_str(&format!(
                "#[used]\n\
                 #[unsafe(link_section = \".tests_data\")]\n\
                 static {}: [u8; {}] = *include_bytes!(\"{}\");\n\n",
                file_name.to_ascii_uppercase(), data.len(), path.to_str().unwrap())
            );
            let meta_info = MetaInfo {
                test_name: String::from(file_name),
                file_size: data.len(),
            };
            meta_infos.push(meta_info);
        }
    }

    let meta_info_array = meta_infos
        .iter()
        .map(|info| format!(
            "MetaInfo {{ test_name: \"{}\", file_size: {} }}",
            info.test_name, info.file_size
        ))
        .collect::<Vec<_>>()
        .join(",\n    ");

    content.push_str(&format!(
        "\n#[derive(Debug)]\n\
         pub struct MetaInfo {{\n\
             pub test_name: &'static str,\n\
             pub file_size: usize,\n\
         }}\n\n\
         pub static TEST_META_INFOS: &[MetaInfo] = &[\n    {}\n];",
        meta_info_array
    ));

    let dest_path = PathBuf::from(env::var("GEN_PATH").unwrap());
    fs::write(
        &dest_path,
        content
    ).expect("Failed to write data.rs");

    println!("cargo:rerun-if-changed=always");
}