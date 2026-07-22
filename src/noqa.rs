use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum NoqaDirective {
    All,
    Specific(Vec<String>),
}

impl NoqaDirective {
    pub fn suppresses(&self, rule_id: &str) -> bool {
        match self {
            NoqaDirective::All => true,
            NoqaDirective::Specific(rules) => rules.iter().any(|r| r == rule_id),
        }
    }
}

/// Parse `# noqa` comments from source text.
/// Returns a mapping from 1-based line numbers to suppression directives.
pub fn parse_noqa_comments(source: &str) -> HashMap<usize, NoqaDirective> {
    let mut map = HashMap::new();
    for (i, line) in source.lines().enumerate() {
        if let Some(directive) = parse_line(line) {
            map.insert(i + 1, directive);
        }
    }
    map
}

fn parse_line(line: &str) -> Option<NoqaDirective> {
    let comment_pos = line.find('#')?;
    let after_hash = &line[comment_pos + 1..];

    let trimmed = after_hash.trim();
    if !trimmed.starts_with("noqa") {
        return None;
    }

    let rest = &trimmed["noqa".len()..];

    if rest.is_empty() {
        return Some(NoqaDirective::All);
    }

    let rest = rest.trim_start();
    let rules_part = rest.strip_prefix(':')?.trim();

    if rules_part.is_empty() {
        return Some(NoqaDirective::All);
    }

    let rules: Vec<String> = rules_part
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();

    if rules.is_empty() {
        Some(NoqaDirective::All)
    } else {
        Some(NoqaDirective::Specific(rules))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source() {
        let map = parse_noqa_comments("");
        assert!(map.is_empty());
    }

    #[test]
    fn no_comments() {
        let source = "log.info('hello')\nlog.info('world')\n";
        let map = parse_noqa_comments(source);
        assert!(map.is_empty());
    }

    #[test]
    fn noqa_suppress_all() {
        let source = "log.info('hello')  # noqa\n";
        let map = parse_noqa_comments(source);
        assert_eq!(map.len(), 1);
        assert!(matches!(map.get(&1).unwrap(), NoqaDirective::All));
    }

    #[test]
    fn noqa_specific_rule() {
        let source = "log.info('hello')  # noqa: SL001\n";
        let map = parse_noqa_comments(source);
        assert_eq!(map.len(), 1);
        let directive = map.get(&1).unwrap();
        assert!(matches!(directive, NoqaDirective::Specific(_)));
        assert!(directive.suppresses("SL001"));
        assert!(!directive.suppresses("SL002"));
    }

    #[test]
    fn noqa_multiple_rules() {
        let source = "log.info('hello')  # noqa: SL001, SL002\n";
        let map = parse_noqa_comments(source);
        assert_eq!(map.len(), 1);
        let directive = map.get(&1).unwrap();
        assert!(directive.suppresses("SL001"));
        assert!(directive.suppresses("SL002"));
        assert!(!directive.suppresses("SL003"));
    }

    #[test]
    fn noqa_no_space_after_hash() {
        let source = "log.info('hello')  #noqa: SL001\n";
        let map = parse_noqa_comments(source);
        assert_eq!(map.len(), 1);
        let directive = map.get(&1).unwrap();
        assert!(directive.suppresses("SL001"));
    }

    #[test]
    fn noqa_extra_space_after_hash() {
        let source = "log.info('hello')  #  noqa: SL001\n";
        let map = parse_noqa_comments(source);
        assert_eq!(map.len(), 1);
        let directive = map.get(&1).unwrap();
        assert!(directive.suppresses("SL001"));
    }

    #[test]
    fn noqa_no_space_after_colon() {
        let source = "log.info('hello')  # noqa:SL001\n";
        let map = parse_noqa_comments(source);
        assert_eq!(map.len(), 1);
        let directive = map.get(&1).unwrap();
        assert!(directive.suppresses("SL001"));
    }

    #[test]
    fn noqa_uppercase_rules() {
        let source = "log.info('hello')  # noqa: sl001, sl002\n";
        let map = parse_noqa_comments(source);
        let directive = map.get(&1).unwrap();
        assert!(directive.suppresses("SL001"));
        assert!(directive.suppresses("SL002"));
    }

    #[test]
    fn noqa_empty_colon_is_all() {
        let source = "log.info('hello')  # noqa:\n";
        let map = parse_noqa_comments(source);
        let directive = map.get(&1).unwrap();
        assert!(matches!(directive, NoqaDirective::All));
    }

    #[test]
    fn noqa_just_colon_and_space_is_all() {
        let source = "log.info('hello')  # noqa: \n";
        let map = parse_noqa_comments(source);
        let directive = map.get(&1).unwrap();
        assert!(matches!(directive, NoqaDirective::All));
    }

    #[test]
    fn noqa_not_matched_in_middle_of_comment() {
        let source = "log.info('hello')  # this is not noqa\n";
        let map = parse_noqa_comments(source);
        assert!(map.is_empty());
    }

    #[test]
    fn noqa_not_matched_if_noqa_is_not_start() {
        let source = "log.info('hello')  # some noqa here\n";
        let map = parse_noqa_comments(source);
        assert!(map.is_empty());
    }

    #[test]
    fn noqa_multiple_lines() {
        let source = "log.info('a')  # noqa: SL001\nlog.info('b')  # noqa: SL002\n";
        let map = parse_noqa_comments(source);
        assert_eq!(map.len(), 2);
        assert!(map.get(&1).unwrap().suppresses("SL001"));
        assert!(map.get(&2).unwrap().suppresses("SL002"));
    }

    #[test]
    fn noqa_single_rule_multiple_commas() {
        let source = "log.info('hello')  # noqa: SL001,,\n";
        let map = parse_noqa_comments(source);
        let directive = map.get(&1).unwrap();
        assert!(directive.suppresses("SL001"));
    }

    #[test]
    fn noqa_with_extra_whitespace() {
        let source = "log.info('hello')  # noqa : SL001\n";
        let map = parse_noqa_comments(source);
        let directive = map.get(&1).unwrap();
        assert!(directive.suppresses("SL001"));
    }
}
