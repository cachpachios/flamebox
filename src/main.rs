use std::path::PathBuf;

use images::docker_io_oauth;

mod fsutils;
mod images;

fn main() {
    let i = images::ImageStore::new(PathBuf::from("imgs/"));
    let auth = docker_io_oauth("repository", "library/python", &["pull"]).expect("Unable to auth.");
    println!("Authenticated with {}", auth);

    let (manifest, root_folder) = i
        .pull_image("python:3.12", Some(&auth))
        .expect("Unable to pull image");
    println!("Pulled image to {:?}", root_folder);
    fsutils::build_squash_fs(&root_folder, &PathBuf::from("root.sfs"))
        .expect("Unable to build squashfs");
    println!("Built squashfs");
}
