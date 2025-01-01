use std::path::Path;

use oci_spec::image::{Arch, ImageManifest, MediaType, Os};
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

pub fn pull_extract_image(
    output_folder: &Path,
    reference: &str,
    auth_token: Option<&str>,
) -> Result<ImageManifest, ImageErrors> {
    let reference =
        Reference::try_from(reference).map_err(|_| ImageErrors::BadlyFormattedReferenceString)?;

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

    println!(
        "Pulled manifest: {} for {}",
        compatible_manifest.digest(),
        reference.repository()
    );

    for (i, layer) in manifest.layers().iter().enumerate() {
        println!(
            "Downloading and extracting layer {}/{}: {} (Size: {})",
            i + 1,
            manifest.layers().len(),
            layer.digest(),
            layer.size()
        );
        let blob_url = format!(
            "https://{}/v2/{}/blobs/{}",
            reference.resolve_registry(),
            reference.repository(),
            layer.digest()
        );

        let mut blob_resp = get(&blob_url, auth_token).map_err(|_| ImageErrors::NetworkError)?;
        extract_layer(&mut blob_resp, &output_folder, layer.media_type())?;
    }

    Ok(manifest)
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

fn extract_layer(
    blob: &mut impl std::io::Read,
    output_folder: &Path,
    media_type: &MediaType,
) -> Result<(), ImageErrors> {
    let reader = match media_type {
        MediaType::ImageLayerGzip => {
            let reader = flate2::read::GzDecoder::new(blob);
            Box::new(reader) as Box<dyn std::io::Read>
        }
        MediaType::ImageLayerZstd => {
            let reader = flate2::read::ZlibDecoder::new(blob);
            Box::new(reader) as Box<dyn std::io::Read>
        }
        MediaType::ImageLayer => Box::new(blob) as Box<dyn std::io::Read>,
        _ => {
            return Err(ImageErrors::IOErr(
                "Unsupported media type for layer in manifest.",
            ))
        }
    };

    let mut tar = tar::Archive::new(reader);
    tar.set_overwrite(true);
    tar.unpack(output_folder).unwrap();
    // .map_err(|_| ImageErrors::IOErr("Unable to extract layer."))?;
    Ok(())
}
