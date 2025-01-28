use log::warn;
use std::cmp::Ordering;
use std::fs;
use std::path::Path;

pub enum PatternType {
    Include,
    Exclude,
    Intersection,
}

impl PartialEq for PatternType {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for PatternType {}

impl PartialOrd for PatternType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PatternType {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (PatternType::Include, PatternType::Include) => Ordering::Equal,
            (PatternType::Include, _) => Ordering::Less,

            (PatternType::Intersection, PatternType::Include) => Ordering::Greater,
            (PatternType::Intersection, PatternType::Intersection) => Ordering::Equal,
            (PatternType::Intersection, PatternType::Exclude) => Ordering::Less,

            (PatternType::Exclude, PatternType::Exclude) => Ordering::Equal,
            (PatternType::Exclude, _) => Ordering::Greater,
        }
    }
}

pub struct PatternResolver;

impl PatternResolver {
    pub fn get_pattern_priority(pattern: &str) -> PatternType {
        match pattern.chars().next() {
            Some('!') => PatternType::Exclude,
            Some('&') => PatternType::Intersection,
            _ => PatternType::Include,
        }
    }

    pub fn resolve_patterns(patterns: &[String]) -> Vec<String> {
        let mut resolved_patterns = Vec::new();

        for pattern in patterns {
            if let Some(file_patterns) = PatternResolver::read_patterns_from_file(pattern) {
                resolved_patterns.extend(file_patterns);
            } else {
                resolved_patterns.push(pattern.clone());
            }
        }

        resolved_patterns
    }

    pub fn resolve_and_sort_patterns(patterns: &[String]) -> Vec<String> {
        let mut resolved_patterns = PatternResolver::resolve_patterns(patterns);

        if resolved_patterns
            .iter()
            .all(|p| p.starts_with('!') || p.starts_with('&'))
        {
            resolved_patterns.push("all".to_string());
        }

        resolved_patterns.sort_by(|a, b| {
            PatternResolver::get_pattern_priority(a).cmp(&PatternResolver::get_pattern_priority(b))
        });

        resolved_patterns
    }

    pub fn read_patterns_from_file(pattern: &str) -> Option<Vec<String>> {
        if !pattern.starts_with('@') {
            return None;
        }

        let filename = &pattern[1..];
        let path = Path::new(filename);

        if !path.exists() || !path.is_file() {
            warn!(
                "Pattern '{}' references a file that doesn't exist: {}",
                pattern, filename
            );
            return None;
        }

        match fs::read_to_string(path) {
            Ok(content) => {
                let lines = content
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect();
                Some(lines)
            }
            Err(err) => {
                warn!("Could not read file '{}': {}", filename, err);
                None
            }
        }
    }
}
