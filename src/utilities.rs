pub fn is_valid_url(url: &str) -> bool {
    let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    else {
        return false;
    };
    let host = rest.split('/').next().unwrap_or("");
    !host.is_empty() && host.contains('.')
}

pub fn minimize_url(full_url: &str) -> String {
    full_url
        .split('/')
        .nth(2)
        .map(|domain| domain.to_string())
        .unwrap_or_else(|| full_url.to_string())
}
