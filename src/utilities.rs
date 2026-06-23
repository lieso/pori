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

pub fn to_safe_dir_name(input: &str) -> String {
    let mut safe_name: String = input
        .chars()
        .map(|c| match c {
            '/' | ':' | '\0' => '_',
            _ => c,
        })
        .collect();

    if safe_name.is_empty() || safe_name == "." || safe_name == ".." {
        safe_name = String::from("unnamed_dir");
    }

    if safe_name.len() > 255 {
        let mut truncated = String::with_capacity(255);
        for c in safe_name.chars() {
            if truncated.len() + c.len_utf8() > 255 {
                break;
            }
            truncated.push(c);
        }
        safe_name = truncated;
    }

    safe_name
}
