use crate::model::{Bookmark, DuplicateRules, SortDirection, SortField};
use pinyin::ToPinyin;
use std::collections::HashMap;
use url::Url;

pub fn sort_bookmarks(
    items: &mut [Bookmark],
    field: SortField,
    direction: SortDirection,
    use_pinyin: bool,
) {
    items.sort_by(|a, b| {
        let ordering = match field {
            SortField::Title => compare_text(&a.title, &b.title, use_pinyin),
            SortField::Tags => compare_text(&a.tags.join(", "), &b.tags.join(", "), use_pinyin),
            SortField::Url => a.url.to_lowercase().cmp(&b.url.to_lowercase()),
        };

        match direction {
            SortDirection::Asc => ordering,
            SortDirection::Desc => ordering.reverse(),
        }
    });
}

fn compare_text(a: &str, b: &str, use_pinyin: bool) -> std::cmp::Ordering {
    if use_pinyin {
        pinyin_key(a).cmp(&pinyin_key(b))
    } else {
        a.to_lowercase().cmp(&b.to_lowercase())
    }
}

fn pinyin_key(input: &str) -> String {
    input
        .chars()
        .map(|character| {
            character
                .to_pinyin()
                .map(|p| p.plain().to_lowercase())
                .unwrap_or_else(|| character.to_lowercase().collect())
        })
        .collect::<Vec<String>>()
        .join("")
}

pub fn normalized_url(raw: &str, rules: &DuplicateRules) -> String {
    // Exact byte-for-byte URL comparison is the safe default.
    if rules.is_strict() {
        return raw.to_string();
    }

    let mut value = raw.to_string();
    if let Ok(mut parsed) = Url::parse(&value) {
        if rules.ignore_fragment {
            parsed.set_fragment(None);
        }
        if rules.ignore_query {
            parsed.set_query(None);
        }
        if rules.ignore_scheme {
            let _ = parsed.set_scheme("http");
        }
        if rules.ignore_www {
            if let Some(host) = parsed.host_str().map(str::to_string) {
                if let Some(without_www) = host.strip_prefix("www.") {
                    let _ = parsed.set_host(Some(without_www));
                }
            }
        }
        if rules.lowercase_host {
            if let Some(host) = parsed.host_str().map(str::to_lowercase) {
                let _ = parsed.set_host(Some(&host));
            }
        }
        value = parsed.to_string();
    }

    if rules.ignore_trailing_slash {
        while value.ends_with('/') && value.matches('/').count() > 2 {
            value.pop();
        }
    }
    value
}

pub fn duplicate_groups(items: &[Bookmark], rules: &DuplicateRules) -> Vec<Vec<usize>> {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();

    for (index, bookmark) in items.iter().enumerate() {
        groups
            .entry(normalized_url(&bookmark.url, rules))
            .or_default()
            .push(index);
    }

    let mut duplicates: Vec<Vec<usize>> =
        groups.into_values().filter(|group| group.len() > 1).collect();
    duplicates.sort_by_key(|group| group[0]);
    duplicates
}

pub fn duplicate_indices(items: &[Bookmark], rules: &DuplicateRules) -> Vec<usize> {
    let mut indices: Vec<usize> = duplicate_groups(items, rules)
        .into_iter()
        .flatten()
        .collect();
    indices.sort_unstable();
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bookmark(id: u64, title: &str, url: &str) -> Bookmark {
        Bookmark {
            id,
            title: title.to_string(),
            url: url.to_string(),
            tags: vec![],
            folders: vec![],
            add_date: None,
            last_modified: None,
            icon: None,
        }
    }

    #[test]
    fn strict_duplicate_detection_requires_exact_url() {
        let items = vec![
            bookmark(1, "one", "https://a.test/x"),
            bookmark(2, "two", "https://a.test/x"),
            bookmark(3, "three", "http://a.test/x"),
            bookmark(4, "four", "https://a.test/x/"),
        ];
        let groups = duplicate_groups(&items, &DuplicateRules::default());
        assert_eq!(groups, vec![vec![0, 1]]);
    }

    #[test]
    fn fragment_rule_merges_urls() {
        let items = vec![
            bookmark(1, "one", "https://a.test/x#one"),
            bookmark(2, "two", "https://a.test/x#two"),
        ];
        let mut rules = DuplicateRules::default();
        rules.ignore_fragment = true;
        assert_eq!(duplicate_groups(&items, &rules).len(), 1);
    }

    #[test]
    fn title_sort_is_case_insensitive() {
        let mut items = vec![
            bookmark(1, "zebra", "https://z.test"),
            bookmark(2, "Alpha", "https://a.test"),
        ];
        sort_bookmarks(
            &mut items,
            SortField::Title,
            SortDirection::Asc,
            false,
        );
        assert_eq!(items[0].title, "Alpha");
    }
}
