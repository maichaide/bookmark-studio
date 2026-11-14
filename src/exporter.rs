use crate::model::Bookmark;
use std::collections::BTreeMap;

#[derive(Default)]
struct FolderNode {
    bookmarks: Vec<Bookmark>,
    children: BTreeMap<String, FolderNode>,
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn export_bookmarks_html(items: &[Bookmark]) -> String {
    let mut root = FolderNode::default();

    for bookmark in items {
        let mut node = &mut root;
        for folder in &bookmark.folders {
            node = node.children.entry(folder.clone()).or_default();
        }
        node.bookmarks.push(bookmark.clone());
    }

    let mut output = String::from(
        "<!DOCTYPE NETSCAPE-Bookmark-file-1>\n\
         <META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n\
         <TITLE>Bookmarks</TITLE>\n\
         <H1>Bookmarks</H1>\n\
         <DL><p>\n",
    );
    render_node(&root, 1, &mut output);
    output.push_str("</DL><p>\n");
    output
}

fn render_node(node: &FolderNode, depth: usize, output: &mut String) {
    let indent = "    ".repeat(depth);

    for (name, child) in &node.children {
        output.push_str(&format!(
            "{indent}<DT><H3>{}</H3>\n{indent}<DL><p>\n",
            escape_html(name)
        ));
        render_node(child, depth + 1, output);
        output.push_str(&format!("{indent}</DL><p>\n"));
    }

    for bookmark in &node.bookmarks {
        output.push_str(&format!(
            "{indent}<DT><A HREF=\"{}\"",
            escape_html(&bookmark.url)
        ));

        if let Some(value) = &bookmark.add_date {
            output.push_str(&format!(" ADD_DATE=\"{}\"", escape_html(value)));
        }
        if let Some(value) = &bookmark.last_modified {
            output.push_str(&format!(
                " LAST_MODIFIED=\"{}\"",
                escape_html(value)
            ));
        }
        if !bookmark.tags.is_empty() {
            output.push_str(&format!(
                " TAGS=\"{}\"",
                escape_html(&bookmark.tags.join(","))
            ));
        }
        if let Some(value) = &bookmark.icon {
            output.push_str(&format!(" ICON=\"{}\"", escape_html(value)));
        }

        output.push_str(&format!(">{}</A>\n", escape_html(&bookmark.title)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_escapes_and_keeps_folder_hierarchy() {
        let bookmark = Bookmark {
            id: 1,
            title: "A & B".to_string(),
            url: "https://a.test/?x=1&y=2".to_string(),
            tags: vec!["one".to_string()],
            folders: vec!["Folder".to_string(), "Nested".to_string()],
            add_date: None,
            last_modified: None,
            icon: None,
        };

        let html = export_bookmarks_html(&[bookmark]);
        assert!(html.contains("<H3>Folder</H3>"));
        assert!(html.contains("<H3>Nested</H3>"));
        assert!(html.contains("A &amp; B"));
        assert!(html.contains("x=1&amp;y=2"));
    }
}
