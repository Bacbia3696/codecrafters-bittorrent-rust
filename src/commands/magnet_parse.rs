use url::Url;

pub fn magnet_parse(magnet_link: &str) -> Result<(), String> {
    // Parse the magnet link URL
    let url = Url::parse(magnet_link)
        .map_err(|e| format!("Invalid magnet link: {}", e))?;

    // Verify it's a magnet link
    if url.scheme() != "magnet" {
        return Err(format!("Not a magnet link: {}", url.scheme()));
    }

    // Extract query parameters
    let mut info_hash = None;
    let mut tracker_urls = Vec::new();

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "xt" => {
                // Exact topic - should be urn:btih:<hash>
                if let Some(hash) = value.strip_prefix("urn:btih:") {
                    info_hash = Some(hash.to_string());
                }
            }
            "dn" => {
                // Display name (parsed but not used in output)
                let _ = value;
            }
            "tr" => {
                // Tracker URL
                tracker_urls.push(value.to_string());
            }
            _ => {
                // Ignore other parameters
            }
        }
    }

    // xt is required
    let info_hash = info_hash.ok_or("Missing required 'xt' parameter (info hash)")?;

    // Print results (in expected order)
    if tracker_urls.is_empty() {
        println!("Tracker URL: (none)");
    } else {
        for tracker in &tracker_urls {
            println!("Tracker URL: {}", tracker);
        }
    }

    println!("Info Hash: {}", info_hash);

    Ok(())
}
