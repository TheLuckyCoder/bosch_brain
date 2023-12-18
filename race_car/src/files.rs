use std::path::PathBuf;

fn get_home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir().expect("Failed to get current working directory"))
}

pub fn get_car_dir() -> PathBuf {
    let mut path = get_home_dir();
    path.push("race_car");
    path
}

pub fn get_car_file(file_name: impl AsRef<str>) -> PathBuf {
    let mut path = get_car_dir();
    path.push(file_name.as_ref());
    path
}
