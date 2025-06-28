pub struct Helper;

impl Helper {
    /// Manual string unescaping for common escape sequences
    pub fn manual_unescape(input: &str) -> String {
        let mut result = input.trim();

        // Remove surrounding double quotes if present
        if result.starts_with('"') && result.ends_with('"') && result.len() > 1 {
            result = &result[1..result.len() - 1];
        }

        // Now unescape the inner content
        result
            .replace("\\r\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\\"", "\"")
            .replace("\\'", "'")
            .replace("\\\\", "\\")
            .replace("\\u0020", " ")
            .replace("\\u0022", "\"")
            .replace("\\u003C", "<")
            .replace("\\u003E", ">")
            .replace("\\u003D", "=")
            .replace("\\u002F", "/")
    }
}
