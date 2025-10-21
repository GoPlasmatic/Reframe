//! String case conversion utilities
//!
//! Provides functions for converting between different string casing conventions
//! (PascalCase, camelCase, snake_case, kebab-case).

/// Convert package-id-with-dashes → PackageIdWithDashes (PascalCase)
///
/// # Examples
/// ```
/// use reframe::utils::string_case::to_pascal_case;
///
/// assert_eq!(to_pascal_case("swift-cbpr-mt-mx"), "SwiftCbprMtMx");
/// assert_eq!(to_pascal_case("other-package"), "OtherPackage");
/// assert_eq!(to_pascal_case("single"), "Single");
/// ```
pub fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut result = first.to_uppercase().collect::<String>();
                    result.push_str(&chars.as_str().to_lowercase());
                    result
                }
            }
        })
        .collect()
}

/// Convert snake_case_field → snakeCaseField (camelCase)
///
/// # Examples
/// ```
/// use reframe::utils::string_case::to_camel_case;
///
/// assert_eq!(to_camel_case("transaction_risk_score"), "transactionRiskScore");
/// assert_eq!(to_camel_case("is_cross_border"), "isCrossBorder");
/// assert_eq!(to_camel_case("single"), "single");
/// ```
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, ch) in s.chars().enumerate() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push_str(&ch.to_uppercase().to_string());
            capitalize_next = false;
        } else if i == 0 {
            result.push_str(&ch.to_lowercase().to_string());
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("swift-cbpr-mt-mx"), "SwiftCbprMtMx");
        assert_eq!(to_pascal_case("other-package"), "OtherPackage");
        assert_eq!(to_pascal_case("single"), "Single");
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(
            to_camel_case("transaction_risk_score"),
            "transactionRiskScore"
        );
        assert_eq!(to_camel_case("is_cross_border"), "isCrossBorder");
        assert_eq!(to_camel_case("single"), "single");
        assert_eq!(to_camel_case(""), "");
    }
}
