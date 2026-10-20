use crate::diff::{DiffReport, Row};
use std::fmt::Write;

pub fn render_json(report: &DiffReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

pub fn render_markdown(report: &DiffReport) -> String {
    let mut output = String::new();
    writeln!(output, "# CSV diff\n").unwrap();
    writeln!(output, "## Summary\n\n{}\n", report.summary).unwrap();
    render_markdown_rows(&mut output, "Added rows", &report.added);
    render_markdown_rows(&mut output, "Removed rows", &report.removed);
    writeln!(output, "## Changed cells\n").unwrap();
    if report.changed.is_empty() {
        writeln!(output, "None\n").unwrap();
    } else {
        writeln!(output, "| Key | Column | Old | New |\n|---|---|---|---|").unwrap();
        for change in &report.changed {
            writeln!(
                output,
                "| {} | {} | {} | {} |",
                escape_markdown(&change.key),
                escape_markdown(&change.column),
                escape_markdown(&change.old),
                escape_markdown(&change.new)
            )
            .unwrap();
        }
        writeln!(output).unwrap();
    }
    render_markdown_columns(&mut output, "Columns added", &report.columns_added);
    render_markdown_columns(&mut output, "Columns removed", &report.columns_removed);
    output
}

pub fn render_table(report: &DiffReport) -> String {
    let mut output = String::new();
    writeln!(output, "Added rows:").unwrap();
    render_table_rows(&mut output, &report.added);
    writeln!(output, "Removed rows:").unwrap();
    render_table_rows(&mut output, &report.removed);
    writeln!(output, "Changed cells:").unwrap();
    if report.changed.is_empty() {
        writeln!(output, "  none").unwrap();
    } else {
        for change in &report.changed {
            writeln!(
                output,
                "  {} / {}: {:?} -> {:?}",
                change.key, change.column, change.old, change.new
            )
            .unwrap();
        }
    }
    writeln!(
        output,
        "Columns added: {}",
        display_list(&report.columns_added)
    )
    .unwrap();
    writeln!(
        output,
        "Columns removed: {}",
        display_list(&report.columns_removed)
    )
    .unwrap();
    writeln!(output, "{}", report.summary).unwrap();
    output
}

fn render_table_rows(output: &mut String, rows: &[Row]) {
    if rows.is_empty() {
        writeln!(output, "  none").unwrap();
    } else {
        for row in rows {
            writeln!(output, "  {}: {:?}", row.key, row.values).unwrap();
        }
    }
}

fn render_markdown_rows(output: &mut String, title: &str, rows: &[Row]) {
    writeln!(output, "## {title}\n").unwrap();
    if rows.is_empty() {
        writeln!(output, "None\n").unwrap();
        return;
    }
    writeln!(output, "| Key | Values |\n|---|---|").unwrap();
    for row in rows {
        let values = row
            .values
            .iter()
            .map(|(column, value)| {
                format!("{}: {}", escape_markdown(column), escape_markdown(value))
            })
            .collect::<Vec<_>>()
            .join("<br>");
        writeln!(output, "| {} | {} |", escape_markdown(&row.key), values).unwrap();
    }
    writeln!(output).unwrap();
}

fn render_markdown_columns(output: &mut String, title: &str, columns: &[String]) {
    writeln!(output, "## {title}\n").unwrap();
    if columns.is_empty() {
        writeln!(output, "None\n").unwrap();
    } else {
        for column in columns {
            writeln!(output, "- {}", escape_markdown(column)).unwrap();
        }
        writeln!(output).unwrap();
    }
}

fn escape_markdown(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}

fn display_list(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(", ")
    }
}
