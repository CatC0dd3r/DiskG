use std::{fs, io};
use std::io::ErrorKind;
use std::path::Path;
use dir_size::get_size_in_bytes;
use size::Size;
use sysinfo::Disks;


pub fn get_files_and_dir() -> Result<Vec<String>, std::io::Error> {
    let current_dir = std::env::current_dir()?;
    let mut files = Vec::new();

    let entries = fs::read_dir(&current_dir)?;

    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        files.push(file_name);
    }

    Ok(files)
}


pub fn get_partions() -> io::Result<Vec<String>> {
    let disks = Disks::new_with_refreshed_list();
    let mut disks_names = vec![];

    for disk in disks.list() {
        disks_names.push(disk.name().to_string_lossy().into_owned());
    }

    Ok(disks_names)
}


pub fn disk_check(disk_name: &str) -> Result<String, std::io::Error> {
    let disks = Disks::new_with_refreshed_list();
    const GIBIBYTE: u64 = 1024 * 1024 * 1024;

    for disk in disks.list() {
        if disk.name().to_string_lossy() == disk_name {
            let total_space_bytes = disk.total_space();
            let available_space_bytes = disk.available_space();
            
            let total_space_gib = total_space_bytes as f64 / GIBIBYTE as f64;
            let available_space_gib = available_space_bytes as f64 / GIBIBYTE as f64;

            let result_string = format!(
                "Имя диска: {:?}\nТочка монтирования: {:?}\nФайловая система: {:?}\nОбщий объём: {:.2} GiB\nДоступный объём: {:.2} GiB",
                disk.name(),
                disk.mount_point(),
                disk.file_system(),
                total_space_gib,
                available_space_gib
            );

            return Ok(result_string);
        }
    }

    Err(io::Error::new(ErrorKind::NotFound, format!("Диск с именем '{}' не найден.", disk_name)))
}


fn convert(bytes: u64) -> String {
    Size::from_bytes(bytes).to_string()
}


pub fn get_size(path_str: &str) -> io::Result<String> {
    let path = Path::new(path_str);
    let metadata = fs::metadata(path)?;

    let item_type = if metadata.is_file() {
        "Файл"
    } else if metadata.is_dir() {
        "Папка"
    } else {
        "Другой объект"
    };

    let name = path.file_name()
                   .and_then(|name| name.to_str())
                   .unwrap_or(path_str);

    let size_bytes = if metadata.is_file() {
        metadata.len()
    } else if metadata.is_dir() {
        get_size_in_bytes(path)? 
    } else {
        0
    };

    let full_size_formatted = convert(size_bytes);

    let result_string = format!(
        "Тип: {}\nИмя: {}\nВес: {}",
        item_type,
        name,
        full_size_formatted
    );

    Ok(result_string)
}