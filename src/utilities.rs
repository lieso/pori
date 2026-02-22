pub fn minimize_url(full_url: &str) -> String {
    full_url
        .split('/')
        .nth(2)
        .map(|domain| domain.to_string())
        .unwrap_or_else(|| full_url.to_string())
}
