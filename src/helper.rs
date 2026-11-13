use slint::SharedString;

pub fn format_size(bytes: i64) -> SharedString {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if bytes >= GB {
        slint::format!("{:>6.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        slint::format!("{:>6.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        slint::format!("{:>6.0} KB", bytes as f64 / KB as f64)
    } else {
        slint::format!("{:>6} B", bytes)
    }
}
