use std::{os, path::PathBuf};

use directories::ProjectDirs;
use std::panic;

pub fn get_save_dir() {
    if let Some(proj_dirs) = ProjectDirs::from("com", "Justin Inc.", "rustbot") {
        let data_dir = proj_dirs.data_dir();

        println!("{:?}", data_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_save_dir_does_not_panic() {
        // Ensure the function can be called without panicking.
        assert!(panic::catch_unwind(|| get_save_dir()).is_ok());
    }
}
