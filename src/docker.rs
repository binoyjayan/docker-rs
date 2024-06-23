use anyhow::Context;

/// Get auth token
fn get_auth_token(auth_url: &str, reg_url: &str, image: &str) -> anyhow::Result<String> {
    let url = format!(
        "https://{}/token?service={}&scope=repository:library/{}:pull",
        auth_url, reg_url, image
    );
    let response = reqwest::blocking::Client::new()
        .get(&url)
        .send()
        .with_context(|| format!("Failed to fetch auth token for image '{}'", image))?;
    let json_body: serde_json::Value = response.json()?;

    let token = json_body["token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Token not found"))?
        .to_string();

    Ok(token)
}

/// Get docker layer blobs
fn get_blobs(hub: &str, image: &str, version: &str, token: &str) -> anyhow::Result<Vec<String>> {
    let url = format!("https://{}/v2/library/{}/manifests/{}", hub, image, version);
    let response = reqwest::blocking::Client::new()
        .get(&url)
        .bearer_auth(token)
        .send()
        .with_context(|| format!("Failed to fetch blobs for image '{}'", image))?;
    let json_body = response.json::<serde_json::Value>()?;

    let mut blobs: Vec<String> = Vec::new();
    if let Some(fs_layers) = json_body["fsLayers"].as_array() {
        for elem in fs_layers {
            blobs.push(String::from(elem["blobSum"].as_str().unwrap()));
        }
    }

    Ok(blobs)
}

fn write_blobs(
    hub_url: &str,
    image: &str,
    blob: Vec<String>,
    token: &str,
    dir: &str,
) -> anyhow::Result<()> {
    let client = reqwest::blocking::Client::new();

    for blob in blob {
        // println!("writing blob: {}", blob);
        let response = client
            .get(&format!(
                "https://{}/v2/library/{}/blobs/{}",
                hub_url, image, blob
            ))
            .bearer_auth(token)
            .send()
            .with_context(|| format!("Failed to fetch blobs for image '{}'", image))?;

        let body = response.bytes()?.to_vec();
        let decoder = libflate::gzip::Decoder::new(body.as_slice())?;
        let mut archive = tar::Archive::new(decoder);
        archive.set_preserve_permissions(true);
        archive.set_unpack_xattrs(true);
        archive.unpack(dir)?;
    }

    Ok(())
}

pub(crate) fn pull_image(
    auth_url: &str,
    reg_url: &str,
    hub_url: &str,
    image: &str,
    root_dir: &str,
) -> anyhow::Result<()> {
    let split: Vec<&str> = image.split(':').collect();
    let image_name = split[0];
    let mut version = "latest";

    if split.len() > 1 {
        version = split[1];
    }

    let token = get_auth_token(auth_url, reg_url, image_name)?;
    let blobs = get_blobs(hub_url, image_name, version, &token)?;

    write_blobs(hub_url, image_name, blobs, &token, root_dir).unwrap();

    Ok(())
}
