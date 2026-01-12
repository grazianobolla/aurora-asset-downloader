mod errors;
mod models;

use std::{env, fs, path::Path};

const URL: &str = "https://www.xboxunity.net/Resources/Lib";
const MIN_FILE_SIZE: u64 = 1024 * 24;
fn get_cover_id(title_id: &str) -> Result<String, errors::AppError> {
    let url = format!("{}/CoverInfo.php?titleid={}", URL, title_id);
    let res: models::CoversResponse = reqwest::blocking::get(url)?.json()?;
    Ok(res
        .covers
        .get(0)
        .ok_or("No cover Id found")?
        .cover_id
        .to_string())
}

fn get_cover_image_bytes(cover_id: &str) -> Result<bytes::Bytes, errors::AppError> {
    let url = format!("{}/Cover.php?size=large&cid={}", URL, cover_id);
    Ok(reqwest::blocking::get(url)?.bytes()?)
}

fn write_file_creating_dirs<P: AsRef<Path>>(
    filename: P,
    bytes: bytes::Bytes,
) -> std::io::Result<()> {
    let path = filename.as_ref();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, bytes)?;
    Ok(())
}

fn save_bytes_to_file(bytes: bytes::Bytes, filename: &str) -> Result<(), errors::AppError> {
    write_file_creating_dirs(filename, bytes)?;
    Ok(())
}

fn get_media_id_from_dir_name(dir_name: &str) -> Result<String, errors::AppError> {
    let (media_id, _) = dir_name.split_once('_').ok_or("Invalid directory name")?;
    Ok(media_id.to_string())
}

fn get_directory_folders<P: AsRef<Path>>(path: P) -> Result<Vec<String>, errors::AppError> {
    fs::read_dir(path)?
        .map(|entry| {
            let entry = entry?;
            Ok(entry.file_name().to_string_lossy().into_owned())
        })
        .collect()
}

fn filter_directories(
    gamedata_path: &str,
    full_directories: Vec<String>,
) -> Result<Vec<String>, errors::AppError> {
    let mut result = Vec::new();

    for dir in full_directories {
        let folder_path = format!("{}/{}/", gamedata_path, dir);
        let has_large_file = fs::read_dir(&folder_path)?
            .filter_map(Result::ok)
            .any(|entry| {
                entry
                    .metadata()
                    .map(|m| m.file_type().is_file() && m.len() > MIN_FILE_SIZE)
                    .unwrap_or(false)
            });

        if !has_large_file {
            result.push(dir);
        }
    }

    Ok(result)
}

fn process_directories(root_path: &str, directories: Vec<String>) -> Result<(), errors::AppError> {
    for (i, folder_name) in directories.iter().enumerate() {
        let progress = ((i as f64 + 1.0) / directories.len() as f64) * 100.0;
        print!("{}% {}", progress.ceil(), folder_name);

        match process_directory(&root_path, &folder_name) {
            Ok(()) => print!(" Ok"),
            Err(err) => print!(" Error! ({})", err),
        }

        println!()
    }

    Ok(())
}

fn process_directory(root_path: &str, folder_name: &str) -> Result<(), errors::AppError> {
    let media_id = get_media_id_from_dir_name(folder_name)?;
    let dest_file = format!("{}/User/Import/{}/cover.png", root_path, media_id);
    print!(" {}", dest_file);
    let cover_id = get_cover_id(&media_id)?;
    let cover_bin = get_cover_image_bytes(&cover_id)?;
    save_bytes_to_file(cover_bin, dest_file.as_str())?;
    Ok(())
}

fn main() {
    let root_path = env::args()
        .collect::<Vec<String>>()
        .get(1)
        .expect("Missing parameter: aurora dashboard root path")
        .to_string();

    // get all directories
    let gamedata_path = format!("{}/Data/GameData", root_path);
    println!("Reading directories from: {}...", gamedata_path);
    let full_directories =
        get_directory_folders(gamedata_path.as_str()).expect("Couldn't read directory");
    println!("Detected {} directories...", full_directories.len());

    // filter empty directories (the ones we want to download)
    let filtered_directories = filter_directories(gamedata_path.as_str(), full_directories)
        .expect("Couldn't filter directories");

    println!(
        "Found {} game data folders with missing covers!",
        filtered_directories.len()
    );

    // iterate and process
    process_directories(root_path.as_str(), filtered_directories)
        .expect("Couldn't process directories");
}
