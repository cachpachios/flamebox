use flamebox::fsutil;
use flamebox::images::{docker_io_oauth, pull_extract_image};
use std::env;
use std::path::PathBuf;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        println!("Usage: {} <image>", args[0]);
        std::process::exit(1);
    }
    let image_name = args[1].split(":").next().unwrap();
    let resource_name = format!("library/{}", &image_name);

    env_logger::init();
    let auth = docker_io_oauth("repository", &resource_name, &["pull"]).expect("Unable to auth.");
    println!("Authenticated with {}", auth);

    let fs_file = PathBuf::from(format!("{}.ext4", image_name));
    fsutil::mkext4(&fs_file, 1 * 1024 * 1024 * 1024);

    let mount_point = tempfile::tempdir()
        .expect("Unable to create tempdir")
        .into_path();
    fsutil::mount_image(&fs_file, &mount_point);

    let pull_result = pull_extract_image(&mount_point, &args[1], Some(&auth));

    fsutil::unmount(&mount_point);

    let _manifest = pull_result.expect("Unable to pull image");
}
