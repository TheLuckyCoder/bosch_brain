//! Utility functions for managing files

use std::path::PathBuf;

/// Gets the home directory of the user
fn get_home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir().expect("Failed to get current working directory"))
}

/// Gets the directory where the car files are stored
pub fn get_car_dir() -> PathBuf {
    let mut path = get_home_dir();
    path.push("race_car");
    path
}

/// Returns the path to a file in the car directory
pub fn get_car_file(file_name: impl AsRef<str>) -> PathBuf {
    let mut path = get_car_dir();
    path.push(file_name.as_ref());
    path
}
