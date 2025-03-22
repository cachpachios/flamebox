use core::time;
use flamebox::firecracker::JailedCracker;
use std::env;
use std::path::PathBuf;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 5 {
        println!(
            "Usage: {} <jailer> <firecracker> <linux_img> <rootfs>",
            args[0]
        );
        std::process::exit(1);
    }

    env_logger::init();
    let fc = JailedCracker::new(&PathBuf::from(&args[1]), &PathBuf::from(&args[2]));

    std::thread::sleep(time::Duration::from_millis(100));

    fc.set_boot(&PathBuf::from(&args[3]))
        .expect("Unable to set boot");
    fc.set_rootfs(&PathBuf::from(&args[4]))
        .expect("Unable to set rootfs");

    std::thread::sleep(time::Duration::from_millis(100));

    fc.start_vm().expect("Unable to start vm");
    println!("VM Started. Will shutdown in 30s.");

    std::thread::sleep(time::Duration::from_secs(30));

    println!("Shutting down with force!");

    fc.cleanup();
}
