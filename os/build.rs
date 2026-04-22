use std::fs::{File, read_dir};
use std::io::Write;

static APP_LINKER: &str = "src/app_linker.S";
static APP_SOURCES_DIR: &str = "../user/src/bin";

fn main() {
    // 获取当前 target 名称和构建模式
    let target = std::env::var("TARGET").expect("Failed to get TARGET");
    let profile = std::env::var("PROFILE").expect("Failed to get PROFILE");
    // 构建用户程序
    let target_path = format!("../user/target/{}/{}", target, profile);
    // 当用户程序发生变化时，重新构建内核
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed={}", target_path);
    // 当 app_linker 模板发生变化时，重新构建内核
    println!("cargo:rerun-if-changed=./build/");

    create_app_linker(target_path.as_str());
}

static APP_LINKER_TEMPLATE: &str = "build/app_linker.S.template";
static APP_LINKER_DATA_ENTRIES_TEMPLATE: &str = "build/app_linker_data_entries.S.template";

fn create_app_linker(target_path: &str) {
    let mut linker_file = File::create(APP_LINKER).expect("Failed to create app linker file");
    let mut apps: Vec<_> = read_dir(APP_SOURCES_DIR)
        .expect("Failed to read app sources dir")
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry
                .expect("Failed to read dir entry info")
                .file_name()
                .into_string()
                .expect("Failed to convert OsString to String");
            let ext_dot_index = name_with_ext
                .find('.')
                .expect("Can't find file name exit dot");
            name_with_ext.drain(ext_dot_index..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();
    println!("cargo:warning=Found apps: {:?}", apps);

    // 读取模板文件信息
    let app_linker_template =
        std::fs::read_to_string(APP_LINKER_TEMPLATE).expect("Failed to read app linker template");
    let app_linker_data_entries_template =
        std::fs::read_to_string(APP_LINKER_DATA_ENTRIES_TEMPLATE)
            .expect("Failed to read app linker data entries template");
    // 构建 app address 标记和 app entry 部分模板内容
    let mut app_address_entries = String::new();
    let mut app_data_entries = String::new();
    for (i, app) in apps.iter().enumerate() {
        app_address_entries.push_str(format!("    .quad app_{}_start\n", i).as_str());
        app_data_entries.push_str(
            &app_linker_data_entries_template
                .replace("{{APP_INDEX}}", &i.to_string())
                .replace(
                    "{{APP_BIN_PATH}}",
                    format!("{}/{}.bin", target_path, app).as_str(),
                ),
        );
    }
    app_address_entries.push_str(format!(r#"    .quad app_{}_end"#, apps.len() - 1).as_str());
    // 写入到住链接脚本中
    let linker_content = app_linker_template
        .replace("{{APP_COUNT}}", &apps.len().to_string())
        .replace("{{APP_ADDRESS_ENTRIES}}", &app_address_entries)
        .replace("{{APP_DATA_ENTRIES}}", &app_data_entries);
    writeln!(linker_file, "{}", linker_content).expect("Failed to write app linker content");
}
