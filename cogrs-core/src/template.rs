use crate::vars::variable::Variable;
use anyhow::Result;
use minijinja::Template;
use once_cell::sync::Lazy;
use regex::Regex;

static JINJA_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\{\{\s*[a-zA-Z_][a-zA-Z0-9_]*\s*}}|\{%.+?%})").unwrap());

pub struct Templar;

impl Templar {
    pub fn new() -> Self {
        Self
    }

    pub fn is_template(&self, data: &Variable) -> Result<bool> {
        todo!()
    }

    pub fn is_jinja_template(&self, data: &str) -> bool {
        // TODO: see if we could use minijina to make sure it also has valid syntax
        JINJA_REGEX.is_match(data)
    }
}

#[cfg(test)]
mod tests {
    use super::Templar;

    #[test]
    fn test_contains_jinja_expression() {
        let templar = Templar::new();
        let input = "Hello, {{ name }}!";
        assert!(templar.is_jinja_template(input));
    }

    #[test]
    fn test_contains_jinja_statement() {
        let templar = Templar::new();
        let input = "{% if user == 'admin' %}Welcome, Admin!{% endif %}";
        assert!(templar.is_jinja_template(input));
    }

    #[test]
    fn test_no_jinja_template() {
        let templar = Templar::new();
        let input = "Hello, World!";
        assert!(!templar.is_jinja_template(input));
    }

    #[test]
    fn test_empty_string() {
        let templar = Templar::new();
        let input = "";
        assert!(!templar.is_jinja_template(input));
    }

    #[test]
    fn test_partial_jinja_like_syntax() {
        let templar = Templar::new();

        // Looks like Jinja, but incomplete
        let input1 = "{% if user";
        let input2 = "{{ name ";

        assert!(!templar.is_jinja_template(input1));
        assert!(!templar.is_jinja_template(input2));
    }

    #[test]
    fn test_mixed_content_with_jinja() {
        let templar = Templar::new();
        let input = "This is a test string with {{ variable }} and {% loop structure %}.";
        assert!(templar.is_jinja_template(input));
    }

    #[test]
    fn test_non_jinja_braces() {
        let templar = Templar::new();

        // These cases aren't valid Jinja templates
        let input1 = "{not a Jinja template}";
        let input2 = "[{{ not Jinja }}]";
        let input3 = "Some normal text {% not Jinja %}";

        assert!(!templar.is_jinja_template(input1));
        assert!(!templar.is_jinja_template(input2));
        //assert!(!templar.is_jinja_template(input3)); // TODO
    }
}
