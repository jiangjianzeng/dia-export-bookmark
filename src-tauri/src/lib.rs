use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tauri::command;

const DIA_BUNDLE_ID: &str = "company.thebrowser.dia";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserInfo {
    pub name: String,
    pub bookmark_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkNode {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
    pub date_added: Option<String>,
    pub children: Vec<BookmarkNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportResult {
    pub success: bool,
    pub message: String,
}

// ==================== Dia Detection ====================

fn check_app_bundle(path: &Path) -> Result<BrowserInfo, String> {
    let plist_path = path.join("Contents/Info.plist");
    if !plist_path.exists() {
        return Err("不是有效的 macOS 应用包".to_string());
    }

    let plist_data = std::fs::read(&plist_path).map_err(|e| e.to_string())?;
    let value: plist::Value = plist::from_bytes(&plist_data).map_err(|e| e.to_string())?;
    let dict = value.as_dictionary().ok_or("无效的 plist 文件")?;
    let bundle_id = dict
        .get("CFBundleIdentifier")
        .and_then(|v| v.as_string())
        .ok_or("未找到 Bundle ID")?;

    if bundle_id != DIA_BUNDLE_ID {
        return Err(format!("请拖拽 Dia 浏览器图标。检测到: {}", bundle_id));
    }

    let home = dirs::home_dir().ok_or("无法找到主目录")?;
    let bookmark_path = home.join("Library/Application Support/Dia/User Data/Default/Bookmarks");

    Ok(BrowserInfo {
        name: "Dia".to_string(),
        bookmark_path: bookmark_path.to_string_lossy().to_string(),
    })
}

fn find_app_bundle(mut path: &Path) -> Option<&Path> {
    loop {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".app") {
                return Some(path);
            }
        }
        match path.parent() {
            Some(parent) => path = parent,
            None => return None,
        }
    }
}

fn detect_dia_browser(app_path: &str) -> Result<BrowserInfo, String> {
    let path = Path::new(app_path);
    if !path.exists() {
        return Err("路径不存在".to_string());
    }

    // Case 1: path itself or one of its ancestors is an .app bundle
    if let Some(bundle) = find_app_bundle(path) {
        return check_app_bundle(bundle);
    }

    // Case 2: user selected a directory (e.g., /Applications) — search inside
    if path.is_dir() {
        for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let entry_path = entry.path();
            if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".app") {
                    if let Ok(info) = check_app_bundle(&entry_path) {
                        return Ok(info);
                    }
                }
            }
        }
        return Err("在所选目录中未找到 Dia 浏览器".to_string());
    }

    Err("无效的路径".to_string())
}

// ==================== Bookmark Parser ====================

#[derive(Debug, Deserialize)]
struct ChromiumBookmarkRoot {
    roots: HashMap<String, ChromiumBookmarkNode>,
}

#[derive(Debug, Deserialize)]
struct ChromiumBookmarkNode {
    id: String,
    name: String,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    node_type: String,
    url: Option<String>,
    #[serde(rename = "date_added")]
    date_added: Option<String>,
    children: Option<Vec<ChromiumBookmarkNode>>,
}

fn parse_dia_bookmarks(path: &str) -> Result<Vec<BookmarkNode>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let root: ChromiumBookmarkRoot = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for (_, node) in root.roots {
        result.push(convert_chromium_node(&node));
    }
    Ok(result)
}

fn convert_chromium_node(node: &ChromiumBookmarkNode) -> BookmarkNode {
    BookmarkNode {
        id: node.id.clone(),
        name: node.name.clone(),
        url: node.url.clone(),
        date_added: node.date_added.clone(),
        children: node
            .children
            .as_ref()
            .map(|c| c.iter().map(convert_chromium_node).collect())
            .unwrap_or_default(),
    }
}

// ==================== HTML Export ====================

fn generate_html_export(bookmarks: &[BookmarkNode], browser_name: &str) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE NETSCAPE-Bookmark-file-1>\n");
    html.push_str("<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n");
    html.push_str(&format!("<TITLE>{} 书签</TITLE>\n", escape_xml(browser_name)));
    html.push_str(&format!("<H1>{} 书签</H1>\n", escape_xml(browser_name)));
    html.push_str("<DL><p>\n");

    for node in bookmarks {
        write_html_node(&mut html, node, 1);
    }

    html.push_str("</DL><p>\n");
    html
}

fn write_html_node(html: &mut String, node: &BookmarkNode, depth: usize) {
    let indent = "    ".repeat(depth);
    if let Some(url) = &node.url {
        let add_date = node
            .date_added
            .as_ref()
            .and_then(|d| d.parse::<i64>().ok())
            .unwrap_or_else(|| chrono::Utc::now().timestamp());
        html.push_str(&format!(
            "{}<DT><A HREF=\"{}\" ADD_DATE=\"{}\" ICON=\"\">{}</A>\n",
            indent,
            escape_xml(url),
            add_date / 1_000_000,
            escape_xml(&node.name)
        ));
    } else {
        html.push_str(&format!(
            "{}<DT><H3>{}</H3>\n",
            indent,
            escape_xml(&node.name)
        ));
        if !node.children.is_empty() {
            html.push_str(&format!("{}<DL><p>\n", indent));
            for child in &node.children {
                write_html_node(html, child, depth + 1);
            }
            html.push_str(&format!("{}</DL><p>\n", indent));
        }
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ==================== Tauri Commands ====================

#[command]
fn detect_browser(path: String) -> Result<BrowserInfo, String> {
    detect_dia_browser(&path)
}

#[command]
fn parse_bookmarks(path: String) -> Result<Vec<BookmarkNode>, String> {
    parse_dia_bookmarks(&path)
}

#[command]
fn export_bookmarks(
    bookmarks: Vec<BookmarkNode>,
    browser_name: String,
    output_path: String,
) -> Result<ExportResult, String> {
    let html = generate_html_export(&bookmarks, &browser_name);
    std::fs::write(&output_path, html).map_err(|e| e.to_string())?;
    Ok(ExportResult {
        success: true,
        message: format!("已导出到 {}", output_path),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            detect_browser,
            parse_bookmarks,
            export_bookmarks
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
