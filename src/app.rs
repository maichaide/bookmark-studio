use crate::exporter;
use crate::manager;
use crate::model::{Bookmark, DuplicateRules, SortDirection, SortField};
use crate::parser;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::fs;
use std::path::PathBuf;

struct Snapshot {
    bookmarks: Vec<Bookmark>,
}

pub struct BookmarkApp {
    bookmarks: Vec<Bookmark>,
    selected_ids: Vec<u64>,
    duplicate_ids: Vec<u64>,
    rules: DuplicateRules,
    sort_field: SortField,
    sort_direction: SortDirection,
    pinyin_sort: bool,
    search: String,
    tag_edit: String,
    folder_edit: String,
    status: String,
    english: bool,
    show_rules: bool,
    undo: Option<Snapshot>,
    source_path: Option<PathBuf>,
}

impl Default for BookmarkApp {
    fn default() -> Self {
        Self {
            bookmarks: Vec::new(),
            selected_ids: Vec::new(),
            duplicate_ids: Vec::new(),
            rules: DuplicateRules::default(),
            sort_field: SortField::Title,
            sort_direction: SortDirection::Asc,
            pinyin_sort: true,
            search: String::new(),
            tag_edit: String::new(),
            folder_edit: String::new(),
            status: "就绪 / Ready".to_string(),
            english: false,
            show_rules: false,
            undo: None,
            source_path: None,
        }
    }
}

impl BookmarkApp {
    fn tr(&self, chinese: &str, english: &str) -> String {
        if self.english {
            english.to_string()
        } else {
            chinese.to_string()
        }
    }

    fn save_undo(&mut self) {
        self.undo = Some(Snapshot {
            bookmarks: self.bookmarks.clone(),
        });
    }

    fn refresh_duplicates(&mut self) {
        self.duplicate_ids = manager::duplicate_indices(&self.bookmarks, &self.rules)
            .into_iter()
            .filter_map(|index| self.bookmarks.get(index).map(|item| item.id))
            .collect();
    }

    fn import_html(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Bookmark HTML", &["html", "htm"])
            .pick_file()
        else {
            return;
        };

        let content = match fs::read(&path) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(error) => {
                self.status = format!("读取失败 / Read failed: {error}");
                return;
            }
        };

        match parser::parse_bookmarks_html(&content) {
            Ok(bookmarks) => {
                self.bookmarks = bookmarks;
                self.selected_ids.clear();
                self.source_path = Some(path.clone());
                self.refresh_duplicates();
                self.status = format!(
                    "已导入 {} 条书签 / Imported {} bookmarks: {}",
                    self.bookmarks.len(),
                    self.bookmarks.len(),
                    path.display()
                );
            }
            Err(error) => {
                self.status = format!("导入失败 / Import failed: {error}");
            }
        }
    }

    fn export_html(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_file_name("bookmarks.html")
            .add_filter("HTML", &["html"])
            .save_file()
        else {
            return;
        };

        let html = exporter::export_bookmarks_html(&self.bookmarks);
        match fs::write(&path, html) {
            Ok(()) => {
                self.status = format!("已导出 / Exported: {}", path.display());
            }
            Err(error) => {
                self.status = format!("导出失败 / Export failed: {error}");
            }
        }
    }

    fn delete_selected(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.save_undo();
        let before = self.bookmarks.len();
        self.bookmarks
            .retain(|item| !self.selected_ids.contains(&item.id));
        let deleted = before - self.bookmarks.len();
        self.selected_ids.clear();
        self.refresh_duplicates();
        self.status = format!("已删除 {deleted} 条书签 / Deleted {deleted} bookmarks");
    }

    fn delete_duplicate_extras(&mut self) {
        let groups = manager::duplicate_groups(&self.bookmarks, &self.rules);
        if groups.is_empty() {
            self.status = self.tr("没有发现重复书签", "No duplicate bookmarks found");
            return;
        }

        self.save_undo();
        let mut remove_ids = Vec::new();

        for group in groups {
            let keep_index = group
                .iter()
                .copied()
                .find(|index| self.selected_ids.contains(&self.bookmarks[*index].id))
                .unwrap_or(group[0]);

            for index in group {
                if index != keep_index {
                    remove_ids.push(self.bookmarks[index].id);
                }
            }
        }

        self.bookmarks
            .retain(|item| !remove_ids.contains(&item.id));
        self.selected_ids.clear();
        self.refresh_duplicates();
        self.status = format!(
            "已清理重复项，删除 {} 条；每组保留一条 / Removed {} duplicates; one kept per group",
            remove_ids.len(),
            remove_ids.len()
        );
    }

    fn apply_tags(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }

        let tags: Vec<String> = self
            .tag_edit
            .split(',')
            .map(str::trim)
            .filter(|tag| !tag.is_empty())
            .map(str::to_string)
            .collect();

        self.save_undo();
        let selected_count = self.selected_ids.len();
        for bookmark in &mut self.bookmarks {
            if self.selected_ids.contains(&bookmark.id) {
                bookmark.tags = tags.clone();
            }
        }
        self.status = format!(
            "已更新 {} 条书签标签 / Updated tags on {} bookmarks",
            selected_count, selected_count
        );
    }

    fn apply_folders(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }

        let folders: Vec<String> = self
            .folder_edit
            .split('/')
            .map(str::trim)
            .filter(|folder| !folder.is_empty())
            .map(str::to_string)
            .collect();

        self.save_undo();
        let selected_count = self.selected_ids.len();
        for bookmark in &mut self.bookmarks {
            if self.selected_ids.contains(&bookmark.id) {
                bookmark.folders = folders.clone();
            }
        }
        self.status = format!(
            "已更新 {} 条书签文件夹 / Updated folders on {} bookmarks",
            selected_count, selected_count
        );
    }

    fn sort_by(&mut self, field: SortField) {
        if self.sort_field == field {
            self.sort_direction = match self.sort_direction {
                SortDirection::Asc => SortDirection::Desc,
                SortDirection::Desc => SortDirection::Asc,
            };
        } else {
            self.sort_field = field;
            self.sort_direction = SortDirection::Asc;
        }

        manager::sort_bookmarks(
            &mut self.bookmarks,
            self.sort_field,
            self.sort_direction,
            self.pinyin_sort,
        );
    }
}

impl eframe::App for BookmarkApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(context, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("🔖 Bookmark Studio");
                ui.separator();

                if ui.button(self.tr("导入 HTML", "Import HTML")).clicked() {
                    self.import_html();
                }
                if ui.button(self.tr("导出 HTML", "Export HTML")).clicked() {
                    self.export_html();
                }
                if ui.button(self.tr("撤销", "Undo")).clicked() {
                    if let Some(snapshot) = self.undo.take() {
                        self.bookmarks = snapshot.bookmarks;
                        self.selected_ids.clear();
                        self.refresh_duplicates();
                        self.status = self.tr("已撤销上一步操作", "Last operation undone");
                    }
                }
                if ui.button(self.tr("重复规则", "Duplicate rules")).clicked() {
                    self.show_rules = !self.show_rules;
                }
                if ui.button(if self.english { "简体中文" } else { "English" }).clicked() {
                    self.english = !self.english;
                }

                ui.separator();
                ui.label(format!(
                    "{}: {}",
                    self.tr("书签", "Bookmarks"),
                    self.bookmarks.len()
                ));
                ui.label(format!(
                    "{}: {}",
                    self.tr("重复项", "Duplicate entries"),
                    self.duplicate_ids.len()
                ));
            });
        });

        let mut show_rules = self.show_rules;
        if show_rules {
            egui::Window::new(self.tr("重复检测规则", "Duplicate matching rules"))
                .open(&mut show_rules)
                .show(context, |ui| {
                    ui.label(self.tr(
                        "默认严格比较 URL 原文；勾选选项会放宽匹配条件。",
                        "By default URLs are compared exactly. Options relax matching.",
                    ));
                    let label_fragment = self.tr("忽略 #fragment", "Ignore #fragment");
                    let label_query = self.tr("忽略 ?query", "Ignore ?query");
                    let label_trailing_slash = self.tr("忽略末尾斜杠", "Ignore trailing slash");
                    let label_scheme = self.tr("忽略 http / https 差异", "Ignore http / https scheme");
                    let label_www = self.tr("忽略 www 前缀", "Ignore www prefix");
                    let label_lowercase_host = self.tr("主机名忽略大小写", "Case-insensitive host");
                    ui.checkbox(&mut self.rules.ignore_fragment, label_fragment);
                    ui.checkbox(&mut self.rules.ignore_query, label_query);
                    ui.checkbox(&mut self.rules.ignore_trailing_slash, label_trailing_slash);
                    ui.checkbox(&mut self.rules.ignore_scheme, label_scheme);
                    ui.checkbox(&mut self.rules.ignore_www, label_www);
                    ui.checkbox(&mut self.rules.lowercase_host, label_lowercase_host);
                    if ui.button(self.tr("重新检测", "Recheck duplicates")).clicked() {
                        self.refresh_duplicates();
                    }
                });
        }
        self.show_rules = show_rules;

        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            ui.label(&self.status);
        });

        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(230.0)
            .show(context, |ui| {
                ui.heading(self.tr("整理工具", "Tools"));
                ui.label(format!(
                    "{}: {}",
                    self.tr("全部书签", "All bookmarks"),
                    self.bookmarks.len()
                ));
                ui.label(format!(
                    "{}: {}",
                    self.tr("重复书签", "Duplicate bookmarks"),
                    self.duplicate_ids.len()
                ));
                ui.label(format!(
                    "{}: {}",
                    self.tr("已选择", "Selected"),
                    self.selected_ids.len()
                ));
                ui.separator();

                let pinyin_sort_label = self.tr("中文按拼音排序", "Sort Chinese by Pinyin");
                ui.checkbox(&mut self.pinyin_sort, pinyin_sort_label);

                ui.separator();
                ui.label(self.tr("批量编辑标签（逗号分隔）", "Bulk edit tags (comma separated)"));
                ui.text_edit_singleline(&mut self.tag_edit);
                if ui
                    .add_enabled(
                        !self.selected_ids.is_empty(),
                        egui::Button::new(self.tr("应用标签到选中项", "Apply tags to selected")),
                    )
                    .clicked()
                {
                    self.apply_tags();
                }

                ui.add_space(6.0);
                ui.label(self.tr(
                    "批量修改文件夹（/ 分隔层级）",
                    "Bulk edit folder (/ separated)",
                ));
                ui.text_edit_singleline(&mut self.folder_edit);
                if ui
                    .add_enabled(
                        !self.selected_ids.is_empty(),
                        egui::Button::new(
                            self.tr("应用文件夹到选中项", "Apply folder to selected"),
                        ),
                    )
                    .clicked()
                {
                    self.apply_folders();
                }

                ui.separator();
                if ui
                    .add_enabled(
                        !self.selected_ids.is_empty(),
                        egui::Button::new(self.tr("删除选中项", "Delete selected")),
                    )
                    .clicked()
                {
                    self.delete_selected();
                }

                if ui
                    .button(self.tr(
                        "清理重复项（每组保留一条）",
                        "Remove duplicates (keep one per group)",
                    ))
                    .clicked()
                {
                    self.delete_duplicate_extras();
                }

                ui.separator();
                ui.label(self.tr(
                    "清理重复项前，可在每个重复组中勾选想保留的书签；未选择时默认保留该组第一条。",
                    "Before cleanup, select the bookmark to keep in each duplicate group. Otherwise the first item is kept.",
                ));
            });

        egui::CentralPanel::default().show(context, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(self.tr("搜索", "Search"));
                ui.text_edit_singleline(&mut self.search);

                if ui.button(self.tr("标题排序", "Sort title")).clicked() {
                    self.sort_by(SortField::Title);
                }
                if ui.button(self.tr("标签排序", "Sort tags")).clicked() {
                    self.sort_by(SortField::Tags);
                }
                if ui.button(self.tr("URL 排序", "Sort URL")).clicked() {
                    self.sort_by(SortField::Url);
                }
            });
            ui.separator();

            let query = self.search.to_lowercase();
            let rows: Vec<Bookmark> = self
                .bookmarks
                .iter()
                .filter(|bookmark| {
                    query.is_empty()
                        || bookmark.title.to_lowercase().contains(&query)
                        || bookmark.url.to_lowercase().contains(&query)
                        || bookmark.tags.join(",").to_lowercase().contains(&query)
                        || bookmark.folders.join("/").to_lowercase().contains(&query)
                })
                .cloned()
                .collect();

            let label_select = self.tr("选择", "Select");
            let label_title = self.tr("标题", "Title");
            let label_tags = self.tr("标签", "Tags");
            let label_folder = self.tr("文件夹", "Folder");

            let all_filtered_selected = !rows.is_empty()
                && rows
                    .iter()
                    .all(|bookmark| self.selected_ids.contains(&bookmark.id));

            let default_text_color = ui.visuals().text_color();

            egui::ScrollArea::horizontal()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::exact(48.0))
                        .column(Column::initial(220.0).at_least(80.0).resizable(true))
                        .column(Column::initial(140.0).at_least(60.0).resizable(true))
                        .column(Column::initial(160.0).at_least(60.0).resizable(true))
                        .column(Column::initial(360.0).at_least(100.0).resizable(true))
                        .header(24.0, |mut header| {
                            header.col(|ui| {
                                let mut select_all = all_filtered_selected;
                                if ui
                                    .checkbox(&mut select_all, "")
                                    .on_hover_text(&label_select)
                                    .changed()
                                {
                                    if select_all {
                                        for bookmark in &rows {
                                            if !self.selected_ids.contains(&bookmark.id) {
                                                self.selected_ids.push(bookmark.id);
                                            }
                                        }
                                    } else {
                                        let filtered_ids: Vec<u64> =
                                            rows.iter().map(|b| b.id).collect();
                                        self.selected_ids
                                            .retain(|id| !filtered_ids.contains(id));
                                    }
                                }
                            });
                            header.col(|ui| {
                                ui.strong(&label_title);
                            });
                            header.col(|ui| {
                                ui.strong(&label_tags);
                            });
                            header.col(|ui| {
                                ui.strong(&label_folder);
                            });
                            header.col(|ui| {
                                ui.strong("URL");
                            });
                        })
                        .body(|body| {
                            body.rows(22.0, rows.len(), |mut row| {
                                let bookmark = &rows[row.index()];
                                let is_duplicate =
                                    self.duplicate_ids.contains(&bookmark.id);
                                let color = if is_duplicate {
                                    egui::Color32::from_rgb(220, 65, 65)
                                } else {
                                    default_text_color
                                };

                                row.col(|ui| {
                                    let mut checked =
                                        self.selected_ids.contains(&bookmark.id);
                                    if ui.checkbox(&mut checked, "").changed() {
                                        if checked {
                                            self.selected_ids.push(bookmark.id);
                                        } else {
                                            self.selected_ids
                                                .retain(|id| *id != bookmark.id);
                                        }
                                    }
                                });
                                row.col(|ui| {
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&bookmark.title).color(color),
                                        )
                                        .truncate(),
                                    )
                                    .on_hover_text(&bookmark.title);
                                });
                                row.col(|ui| {
                                    let tags_text = bookmark.tags.join(", ");
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&tags_text).color(color),
                                        )
                                        .truncate(),
                                    )
                                    .on_hover_text(&tags_text);
                                });
                                row.col(|ui| {
                                    let folder_text = bookmark.folders.join(" / ");
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&folder_text).color(color),
                                        )
                                        .truncate(),
                                    )
                                    .on_hover_text(&folder_text);
                                });
                                row.col(|ui| {
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&bookmark.url).color(color),
                                        )
                                        .truncate(),
                                    )
                                    .on_hover_text(&bookmark.url);
                                });
                            });
                        });
                });
        });
    }
}