use std::path::PathBuf;

use flamebox::fsutil;
use flamebox::images::{docker_io_oauth, pull_extract_image};

fn main() {
    env_logger::init();
    let auth = docker_io_oauth("repository", "library/ubuntu", &["pull"]).expect("Unable to auth.");
    println!("Authenticated with {}", auth);

    let fs_file = PathBuf::from("data/rootfs.ext4");
    fsutil::mkext4(&fs_file, 10 * 1024 * 1024 * 1024);

    let mount_point = tempfile::tempdir()
        .expect("Unable to create tempdir")
        .into_path();
    fsutil::mount_image(&fs_file, &mount_point);

    let pull_result = pull_extract_image(&mount_point, "ubuntu:24.04", Some(&auth));

    fsutil::unmount(&mount_point);

    let _manifest = pull_result.expect("Unable to pull image");
}
