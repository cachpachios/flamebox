use std::path::Path;

use squashfs_ng::write::TreeProcessor;

pub fn build_squash_fs(root: &Path, output: &Path) -> Result<(), String> {
    let mut sfs = TreeProcessor::new(output).map_err(|e| e.to_string())?;
    sfs.process(root).map_err(|e| e.to_string())?;
    Ok(())
}
