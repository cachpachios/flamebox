use std::fs::File;
use std::path::Path;
use std::process::Command;

pub fn mkext4(file: &Path, size: u64) {
    let f = File::create(file).expect("Unable to create file");
    f.set_len(size).expect("Unable to set file size");
    let mut cmd = Command::new("mkfs.ext4");
    cmd.arg(file);
    if !cmd
        .output()
        .expect("Unable to create ext4 filesystem")
        .status
        .success()
    {
        panic!("Unable to create ext4 filesystem");
    }
}

pub fn mount_image(image: &Path, mount_point: &Path) {
    let _ = std::fs::create_dir(mount_point);
    let mut cmd = Command::new("mount");
    cmd.arg("-o").arg("loop").arg(image).arg(mount_point);
    if !cmd
        .output()
        .expect("Unable to mount image")
        .status
        .success()
    {
        panic!("Unable to mount image");
    }
}

pub fn unmount(mount_point: &Path) {
    let mut cmd = Command::new("umount");
    cmd.arg(mount_point);
    if !cmd
        .output()
        .expect("Unable to unmount image")
        .status
        .success()
    {
        panic!("Unable to unmount image");
    }
}
