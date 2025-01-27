use anyhow::bail;
use regex::Regex;

pub fn difference_update_vec<T: PartialEq>(vec: &mut Vec<T>, other: &Vec<T>) {
    // Iterate over elements of `other` - remove them from `vec`
    vec.retain(|item| !other.contains(item));
}

/// Takes a pattern, checks if it has a subscript, and returns the pattern
/// without the subscript and a (start,end) tuple representing the given
/// subscript (or None if there is no subscript).
pub fn split_subscript(pattern: &str) -> anyhow::Result<(String, Option<(i32, Option<i32>)>)> {
    // Do not parse regexes for enumeration info
    if pattern.starts_with('~') {
        return Ok((pattern.to_string(), None));
    }

    // Compiling the regex for pattern with subscript
    let pattern_with_subscript = Regex::new(
        r"(?x)
            ^
            (.+?)                    # A pattern expression ending with...
            \[(?:                   # A [subscript] expression comprising:
                (-?[0-9]+)|         # A single positive or negative number
                ([0-9]*)([:-])      # Or an x:y or x:- range (start can be empty; e.g., :y or :-y).
                ([0-9]*)          # End number (can be empty, can be negative).
            )]
            $
        ",
    )?;

    // Using the regex to validate and parse the input pattern
    if let Some(captures) = pattern_with_subscript.captures(pattern) {
        let trimmed_pattern = captures.get(1).map_or("", |m| m.as_str()).to_string();

        if let Some(idx_match) = captures.get(2) {
            let idx = idx_match.as_str().parse::<i32>()?;
            return Ok((trimmed_pattern, Some((idx, None))));
        } else {
            let start = captures.get(3).map_or(0, |start_str| {
                let s = start_str.as_str();
                if s.is_empty() {
                    0
                } else {
                    s.parse::<i32>().unwrap_or(0)
                }
            });

            let sep = captures.get(4).map_or(":", |m| m.as_str());

            let end = captures.get(5).map_or(-1, |end_str| {
                let s = end_str.as_str();
                if s.is_empty() {
                    -1
                } else {
                    s.parse::<i32>().unwrap_or(-1)
                }
            });

            if sep == "-" {
                println!("Warning: Use [x:y] inclusive subscripts instead of [x-y], which has been removed.");
            }

            return Ok((trimmed_pattern, Some((start, Some(end)))));
        }
    }

    Ok((pattern.to_string(), None))
}

pub fn glob_to_regex(glob: &str) -> anyhow::Result<String> {
    let mut regex = String::from("^");
    for ch in glob.chars() {
        match ch {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' | '\\' | '+' | '(' | ')' | '|' | '^' | '$' | '[' | ']' | '{' | '}' => {
                regex.push('\\');
                regex.push(ch);
            }
            _ => regex.push(ch),
        }
    }
    regex.push('$');
    Ok(regex)
}

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
        let stride = captures
            .get(3)
            .map_or(1, |m| m.as_str().parse::<usize>().unwrap_or(1)); // Default stride is 1

        // Determine if the range is numeric or alphabetic
        if let (Ok(start_num), Ok(end_num)) = (start.parse::<usize>(), end.parse::<usize>()) {
            // Generate numeric range values with the given stride
            for i in (start_num..=end_num).step_by(stride) {
                // Replace the full pattern in the original string with the current value
                let generated_host = pattern.replace(full_match, &i.to_string());
                hosts.push(generated_host);
            }
        } else if let (Some(start_char), Some(end_char)) =
            (start.chars().next(), end.chars().next())
        {
            if start_char.is_alphabetic() && end_char.is_alphabetic() {
                // Generate alphabetic range values with the given stride
                let mut current = start_char as u32;

                while current <= end_char as u32 {
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
    use crate::inventory::manager::InventoryManager;

    #[test]
    fn test_split_subscript_no_subscript() {
        let input_pattern = "pattern_without_subscript";
        let (parsed_pattern, subscript) = split_subscript(input_pattern).unwrap();

        assert_eq!(parsed_pattern, input_pattern.to_string());
        assert!(subscript.is_none());
    }

    #[test]
    fn test_split_subscript_with_single_index_subscript() {
        let input_pattern = "host[3]";
        let (parsed_pattern, subscript) = split_subscript(input_pattern).unwrap();

        assert_eq!(parsed_pattern, "host".to_string());
        assert_eq!(subscript, Some((3, None)));
    }

    #[test]
    fn test_split_subscript_with_positive_range_subscript() {
        let input_pattern = "host[1:4]";
        let (parsed_pattern, subscript) = split_subscript(input_pattern).unwrap();

        assert_eq!(parsed_pattern, "host".to_string());
        assert_eq!(subscript, Some((1, Some(4))));
    }

    #[test]
    fn test_split_subscript_with_negative_index() {
        let input_pattern = "host[-2]";
        let (parsed_pattern, subscript) = split_subscript(input_pattern).unwrap();

        assert_eq!(parsed_pattern, "host".to_string());
        assert_eq!(subscript, Some((-2, None)));
    }

    #[test]
    fn test_split_subscript_with_positive_to_infinite_range() {
        let input_pattern = "host[5:]";
        let (parsed_pattern, subscript) = split_subscript(input_pattern).unwrap();

        assert_eq!(parsed_pattern, "host".to_string());
        assert_eq!(subscript, Some((5, Some(-1))));
    }

    #[test]
    fn test_split_subscript_with_infinite_to_negative_end_range() {
        let input_pattern = "host[:-3]";
        let (parsed_pattern, subscript) = split_subscript(input_pattern).unwrap();

        assert_eq!(parsed_pattern, "host[:-3]".to_string());
        assert_eq!(subscript, None);
    }

    #[test]
    fn test_split_subscript_with_invalid_pattern() {
        let input_pattern = "host[invalid]";
        let (parsed_pattern, subscript) = split_subscript(input_pattern).unwrap();

        // In case of an invalid pattern, the function should return the full pattern unchanged,
        // and subscript should be None.
        assert_eq!(parsed_pattern, input_pattern.to_string());
        assert!(subscript.is_none());
    }

    #[test]
    fn test_split_subscript_with_regex_flag() {
        let input_pattern = "~host_regex[3]";
        let (parsed_pattern, subscript) = split_subscript(input_pattern).unwrap();

        // Regex patterns are not subjected to parsing for subscripts and remain intact.
        assert_eq!(parsed_pattern, input_pattern.to_string());
        assert!(subscript.is_none());
    }

    #[test]
    fn test_split_subscript_edge_cases() {
        // Empty input
        let input_pattern_empty = "";
        let (parsed_empty, subscript_empty) = split_subscript(input_pattern_empty).unwrap();
        assert_eq!(parsed_empty, "".to_string());
        assert!(subscript_empty.is_none());

        // Special characters in pattern
        let input_pattern_special = "host[*][1]";
        let (parsed_special, subscript_special) = split_subscript(input_pattern_special).unwrap();
        assert_eq!(parsed_special, "host[*]".to_string());
        assert_eq!(subscript_special, Some((1, None)));
    }

    #[test]
    fn test_numeric_pattern_suffix() {
        let pattern = "host[0:5]";
        let expected = vec!["host0", "host1", "host2", "host3", "host4", "host5"];
        let result = parse_host_pattern(pattern).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_numeric_pattern_with_stride() {
        let pattern = "host[0:10:2]";
        let expected = vec!["host0", "host2", "host4", "host6", "host8", "host10"];
        let result = parse_host_pattern(pattern).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_numeric_pattern_with_prefix_and_suffix() {
        let pattern = "prefix[0:5].suffix";
        let expected = vec![
            "prefix0.suffix",
            "prefix1.suffix",
            "prefix2.suffix",
            "prefix3.suffix",
            "prefix4.suffix",
            "prefix5.suffix",
        ];
        let result = parse_host_pattern(pattern).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_alphabetic_pattern_suffix() {
        let pattern = "host[a:d]";
        let expected = vec!["hosta", "hostb", "hostc", "hostd"];
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
        assert!(
            result.is_err(),
            "Expected error for invalid numeric pattern"
        );
    }

    #[test]
    fn test_invalid_alphabetic_pattern() {
        let pattern = "prefix[0:z]suffix";
        let result = parse_host_pattern(pattern);
        assert!(
            result.is_err(),
            "Expected error for invalid alphabetic pattern"
        );
    }

    #[test]
    fn test_multiple_patterns_in_string() {
        let pattern = "host[0:3]-region[a:c]";
        let result = parse_host_pattern(pattern);
        assert!(
            result.is_err(),
            "Multiple patterns are not currently supported"
        );
    }
}
