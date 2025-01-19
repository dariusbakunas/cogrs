use anyhow::bail;
use regex::Regex;

pub fn parse_host_pattern(pattern: &str) -> anyhow::Result<Vec<String>> {
    let mut hosts = Vec::new();

    // Match patterns of the form [start:end(:stride)?]
    let re = Regex::new(r"\[([a-zA-Z0-9]+):([a-zA-Z0-9]+)(?::(\d+))?]")?;


    let matches: Vec<_> = re.captures_iter(pattern).collect();
    if matches.len() > 1 {
        // Return an error if multiple patterns are detected
        bail!("Multiple patterns not supported: {}", pattern);
    } else if let Some(captures) = matches.get(0) {
        // Extract the prefix, suffix, and range part from the input
        let full_match = &captures[0];
        let start = &captures[1];
        let end = &captures[2];
        let stride = captures.get(3).map_or(1, |m| m.as_str().parse::<usize>().unwrap_or(1)); // Default stride is 1

        // Determine if the range is numeric or alphabetic
        if let (Ok(start_num), Ok(end_num)) = (start.parse::<usize>(), end.parse::<usize>()) {
            // Generate numeric range values with the given stride
            for i in (start_num..end_num).step_by(stride) {
                // Replace the full pattern in the original string with the current value
                let generated_host = pattern.replace(full_match, &i.to_string());
                hosts.push(generated_host);
            }
        } else if let (Some(start_char), Some(end_char)) = (start.chars().next(), end.chars().next()) {
            if start_char.is_alphabetic() && end_char.is_alphabetic() {
                // Generate alphabetic range values with the given stride
                let mut current = start_char as u32;

                while current < end_char as u32 {
                    if let Some(current_char) = char::from_u32(current) {
                        // Replace the full pattern in the original string with the current character
                        let generated_host = pattern.replace(full_match, &current_char.to_string());
                        hosts.push(generated_host);
                    }
                    current += stride as u32;
                }
            } else {
                bail!("Invalid alphabetic range in pattern: {}", pattern);
            }
        } else {
            bail!("Invalid range in pattern: {}", pattern);
        }
    } else {
        // If no pattern is present, return the input string as-is
        hosts.push(pattern.to_string());
    }

    // Return the generated hostnames
    Ok(hosts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_pattern_suffix() {
        let pattern = "host[0:5]";
        let expected = vec!["host0", "host1", "host2", "host3", "host4"];
        let result = parse_host_pattern(pattern).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_numeric_pattern_with_stride() {
        let pattern = "host[0:10:2]";
        let expected = vec!["host0", "host2", "host4", "host6", "host8"];
        let result = parse_host_pattern(pattern).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_numeric_pattern_with_prefix_and_suffix() {
        let pattern = "prefix[0:5]suffix";
        let expected = vec![
            "prefix0suffix",
            "prefix1suffix",
            "prefix2suffix",
            "prefix3suffix",
            "prefix4suffix",
        ];
        let result = parse_host_pattern(pattern).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_alphabetic_pattern_suffix() {
        let pattern = "host[a:d]";
        let expected = vec!["hosta", "hostb", "hostc"];
        let result = parse_host_pattern(pattern).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_alphabetic_pattern_with_stride() {
        let pattern = "host[a:f:2]";
        let expected = vec!["hosta", "hostc", "hoste"];
        let result = parse_host_pattern(pattern).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pattern_with_no_matches() {
        let pattern = "host";
        let expected = vec!["host"];
        let result = parse_host_pattern(pattern).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_invalid_numeric_pattern() {
        let pattern = "host[0:a]";
        let result = parse_host_pattern(pattern);
        assert!(result.is_err(), "Expected error for invalid numeric pattern");
    }

    #[test]
    fn test_invalid_alphabetic_pattern() {
        let pattern = "prefix[0:z]suffix";
        let result = parse_host_pattern(pattern);
        assert!(result.is_err(), "Expected error for invalid alphabetic pattern");
    }

    #[test]
    fn test_multiple_patterns_in_string() {
        let pattern = "host[0:3]-region[a:c]";
        let result = parse_host_pattern(pattern);
        assert!(result.is_err(), "Multiple patterns are not currently supported");
        // Note: This test assumes multiple patterns in the same string are unsupported.
    }
}
