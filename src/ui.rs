use unicode_width::UnicodeWidthStr;

pub fn print_info_card(title: &str, rows: Vec<(String, String)>) {
    if rows.is_empty() {
        return;
    }

    let label_width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);

    let formatted_rows: Vec<String> = rows
        .into_iter()
        .map(|(label, value)| {
            format!(
                "{:<label_width$} : {}",
                label,
                value,
                label_width = label_width
            )
        })
        .collect();

    let mut inner_width = formatted_rows
        .iter()
        .map(|line| UnicodeWidthStr::width(line.as_str()))
        .max()
        .unwrap_or(0);
    let title_width = UnicodeWidthStr::width(title);
    inner_width = inner_width.max(title_width + 2);
    let total_width = inner_width + 4;

    println!();
    println!("{}", card_top_line(title, total_width));
    for line in formatted_rows {
        let current_width = UnicodeWidthStr::width(line.as_str());
        let padding = " ".repeat(inner_width.saturating_sub(current_width));
        println!("│ {}{} │", line, padding);
    }
    println!("╰{}╯", "─".repeat(total_width.saturating_sub(2)));
    println!();
}

fn card_top_line(title: &str, total_width: usize) -> String {
    let mut line = format!("╭─ {title} ");
    let prefix_width = UnicodeWidthStr::width(line.as_str());
    let remaining = total_width.saturating_sub(prefix_width.saturating_add(1)); // reserve space for closing corner
    line.push_str(&"─".repeat(remaining));
    line.push('╮');
    line
}
