use crate::model::Bookmark;
use kuchiki::{traits::TendrilSink, NodeRef};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn parse_bookmarks_html(input: &str) -> Result<Vec<Bookmark>, String> {
    if input.trim().is_empty() {
        return Err("文件内容为空".to_string());
    }

    let document = kuchiki::parse_html().one(input);
    let mut bookmarks = Vec::new();
    walk(&document, &[], &mut bookmarks);

    if bookmarks.is_empty() {
        return Err("没有找到书签条目。请确认文件是浏览器导出的 bookmarks.html。".to_string());
    }
    Ok(bookmarks)
}

fn element_name(node: &NodeRef) -> Option<String> {
    node.as_element()
        .map(|element| element.name.local.to_string().to_ascii_lowercase())
}

fn attribute(node: &NodeRef, name: &str) -> Option<String> {
    let element = node.as_element()?;
    element.attributes.borrow().get(name).map(str::to_string)
}

fn walk(node: &NodeRef, folders: &[String], output: &mut Vec<Bookmark>) {
    let children: Vec<NodeRef> = node.children().collect();
    let mut pending_folder: Option<String> = None;

    for child in children {
        match element_name(&child).as_deref() {
            Some("h3") => {
                let name = child.text_contents().trim().to_string();
                if !name.is_empty() {
                    pending_folder = Some(name);
                }
            }
            Some("a") => {
                if let Some(url) = attribute(&child, "href") {
                    let title = child.text_contents().trim().to_string();
                    let tags = attribute(&child, "tags")
                        .unwrap_or_default()
                        .split(',')
                        .map(str::trim)
                        .filter(|tag| !tag.is_empty())
                        .map(str::to_string)
                        .collect();

                    output.push(Bookmark {
                        id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
                        title,
                        url,
                        tags,
                        folders: folders.to_vec(),
                        add_date: attribute(&child, "add_date"),
                        last_modified: attribute(&child, "last_modified"),
                        icon: attribute(&child, "icon"),
                    });
                }
            }
            Some("dl") => {
                let mut nested_folders = folders.to_vec();
                if let Some(name) = pending_folder.take() {
                    nested_folders.push(name);
                }
                walk(&child, &nested_folders, output);
            }
            _ => {
                // Netscape bookmark exports commonly wrap entries in DT and P nodes.
                walk(&child, folders, output);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_bookmarks_and_entities() {
        let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
        <META charset="UTF-8"><TITLE>Bookmarks</TITLE>
        <DL><p><DT><H3>开发 &amp; 工具</H3>
        <DL><p><DT><A HREF="https://example.com" ADD_DATE="123" TAGS="dev,rust">示例 &amp; 首页</A>
        <DT><H3>子目录</H3><DL><p><DT><A HREF="https://rust-lang.org">Rust</A></DL>
        </DL></DL>"#;

        let items = parse_bookmarks_html(html).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "示例 & 首页");
        assert_eq!(items[0].folders, vec!["开发 & 工具"]);
        assert_eq!(items[0].tags, vec!["dev", "rust"]);
        assert_eq!(items[1].folders, vec!["开发 & 工具", "子目录"]);
    }

    #[test]
    fn rejects_empty_document() {
        assert!(parse_bookmarks_html("  ").is_err());
    }
}
