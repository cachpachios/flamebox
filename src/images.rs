use std::path::PathBuf;

use oci_spec::image::{Arch, ImageManifest, Os};
use oci_spec::{distribution::Reference, image::ImageIndex};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug)]
pub enum ImageErrors {
    BadlyFormattedReferenceString,
    NetworkError,
    NoCompatibleImageAvailable,
    UnableToParse,
    IOErr(&'static str),
}

pub struct ImageStore {
    root: PathBuf,
}

impl ImageStore {
    pub fn new(root: PathBuf) -> ImageStore {
        ImageStore { root }
    }

    pub fn pull_image(self, reference: &str, auth_token: Option<&str>) -> Result<(), ImageErrors> {
        let reference = Reference::try_from(reference)
            .map_err(|_| ImageErrors::BadlyFormattedReferenceString)?;

        let index_url = format!(
            "https://{}/v2/{}/manifests/{}",
            reference.resolve_registry(),
            reference.repository(),
            reference.tag().unwrap_or("latest"),
        );

        let index: ImageIndex = get(&index_url, auth_token)?
            .json()
            .expect("Cant parse index!");

        let compatible_manifest = index
            .manifests()
            .iter()
            .find(|d| {
                d.platform().as_ref().map_or(false, |p| {
                    *p.architecture() == Arch::Amd64 && *p.os() == Os::Linux
                })
            })
            .ok_or(ImageErrors::NoCompatibleImageAvailable)?;

        let manifest_url = format!(
            "https://{}/v2/{}/manifests/{}",
            reference.resolve_registry(),
            reference.repository(),
            compatible_manifest.digest()
        );

        let manifest: ImageManifest = get(&manifest_url, auth_token)?
            .json()
            .map_err(|_| ImageErrors::UnableToParse)?;

        println!("Pulled manifest: {}", compatible_manifest.digest());

        let folder = self.root.join(reference.repository());
        let _ = std::fs::create_dir_all(&folder);

        let manifest_file = folder.join("manifest.json");
        let file: std::fs::File = std::fs::File::create(manifest_file)
            .map_err(|_| ImageErrors::IOErr("Unable to create manifest file."))?;
        serde_json::to_writer(&file, &manifest)
            .map_err(|_| ImageErrors::IOErr("Unable to write manifest to file."))?;

        let blob_folder = folder.join("blobs");
        let _ = std::fs::create_dir_all(&blob_folder);

        for (i, blob) in manifest.layers().iter().enumerate() {
            println!(
                "Downloading layer {}/{}: {} (Size: {})",
                i + 1,
                manifest.layers().len(),
                blob.digest(),
                blob.size()
            );
            let blob_url = format!(
                "https://{}/v2/{}/blobs/{}",
                reference.resolve_registry(),
                reference.repository(),
                blob.digest()
            );

            let blob_file = blob_folder.join(blob.digest().to_string());
            let file: std::fs::File = std::fs::File::create(blob_file)
                .map_err(|_| ImageErrors::IOErr("Unable to create blob file in FS."))?;

            let blob = get(&blob_url, auth_token)?
                .bytes()
                .map_err(|_| ImageErrors::NetworkError)?;

            std::io::copy(&mut blob.as_ref(), &mut &file)
                .map_err(|_| ImageErrors::IOErr("Unable to write blob data to file."))?;
        }

        Ok(())
    }
}

pub fn docker_io_oauth(
    scope_type: &str,
    resource_name: &str,
    actions: &[&str],
) -> Result<String, String> {
    let url = format!(
        "https://auth.docker.io/token?service=registry.docker.io&scope={}:{}:{}",
        scope_type,
        resource_name,
        actions.join(",")
    );
    let resp = reqwest::blocking::get(url).map_err(|e| e.to_string())?;

    #[derive(Deserialize)]
    struct TokenResponse {
        token: String,
    }
    let resp: TokenResponse = resp.json().map_err(|e| e.to_string())?;

    Ok(resp.token)
}

fn get(url: &str, auth_token: Option<&str>) -> Result<reqwest::blocking::Response, ImageErrors> {
    let client = Client::new();
    let mut request = client.get(url);
    if let Some(token) = auth_token {
        request = request.bearer_auth(token);
    }
    request.send().map_err(|_| ImageErrors::NetworkError)
}
