use std::path::PathBuf;

use images::docker_io_oauth;

mod images;

fn main() {
    let i = images::ImageStore::new(PathBuf::from("imgs/"));
    let auth = docker_io_oauth("repository", "library/python", &["pull"]).expect("Unable to auth.");
    println!("Authenticated with {}", auth);

    i.pull_image("python:3.12", Some(&auth))
        .expect("Unable to pull ubuntu image");
}
