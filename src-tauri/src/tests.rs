#[cfg(test)]
mod replace_tests {
    use crate::engine::{apply_rules, apply_rules_cached, detect_conflicts};
    use crate::state::ReplaceCache;
    use crate::types::ReplaceRule;
    use std::collections::HashMap;

    fn make_rule(id: i64, old: &str, new: &str, enabled: bool, priority: i64) -> ReplaceRule {
        ReplaceRule {
            id,
            old_text: old.to_string(),
            new_text: new.to_string(),
            is_regex: false,
            enabled,
            priority,
            created_at: String::new(),
            updated_at: String::new(),
            conflicts: vec![],
        }
    }

    #[test]
    fn test_sequential_apply() {
        let rules = vec![
            make_rule(1, "A", "B", true, 0),
            make_rule(2, "B", "C", true, 1),
        ];
        assert_eq!(apply_rules("A", &rules), "C");
    }

    #[test]
    fn test_quote_replacement() {
        let rules = vec![
            make_rule(1, "\u{201C}", "'", true, 0),
            make_rule(2, "\u{201D}", "'", true, 1),
            make_rule(3, "\u{2018}", "'", true, 2),
            make_rule(4, "\u{2019}", "'", true, 3),
        ];
        let input = "\u{201C}안녕\u{201D} \u{2018}반갑\u{2019}";
        assert_eq!(apply_rules(input, &rules), "'안녕' '반갑'");
    }

    #[test]
    fn test_disabled_rule_skipped() {
        let rules = vec![
            make_rule(1, "hello", "world", false, 0),
            make_rule(2, "foo", "bar", true, 1),
        ];
        assert_eq!(apply_rules("hello foo", &rules), "hello bar");
    }

    #[test]
    fn test_cache_invalidation_on_version_change() {
        let mut cache = ReplaceCache { ruleset_version: 0, entries: HashMap::new() };
        let rules_v0 = vec![make_rule(1, "A", "B", true, 0)];
        assert_eq!(apply_rules_cached("A", &rules_v0, &mut cache), "B");

        cache.ruleset_version += 1;
        let rules_v1 = vec![make_rule(1, "A", "C", true, 0)];
        assert_eq!(apply_rules_cached("A", &rules_v1, &mut cache), "C");
    }

    #[test]
    fn test_conflict_cycle_detection() {
        let rules = vec![
            make_rule(1, "A", "B", true, 0),
            make_rule(2, "B", "A", true, 1),
        ];
        let conflicts = detect_conflicts(&rules);
        assert!(conflicts.contains_key(&1));
        assert!(conflicts.contains_key(&2));
    }

    #[test]
    fn test_empty_content_returns_empty() {
        let rules = vec![make_rule(1, "A", "B", true, 0)];
        assert_eq!(apply_rules_cached("", &rules, &mut ReplaceCache {
            ruleset_version: 0, entries: HashMap::new()
        }), "");
    }

    #[test]
    fn test_no_match_returns_original() {
        let rules = vec![make_rule(1, "X", "Y", true, 0)];
        assert_eq!(apply_rules("hello", &rules), "hello");
    }
}
