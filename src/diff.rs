use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Row {
    pub key: String,
    pub values: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CellChange {
    pub key: String,
    pub column: String,
    pub old: String,
    pub new: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DiffReport {
    pub added: Vec<Row>,
    pub removed: Vec<Row>,
    pub changed: Vec<CellChange>,
    pub columns_added: Vec<String>,
    pub columns_removed: Vec<String>,
    pub summary: String,
}

impl DiffReport {
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty()
            || !self.removed.is_empty()
            || !self.changed.is_empty()
            || !self.columns_added.is_empty()
            || !self.columns_removed.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffError {
    MissingKeyColumn(String),
    DuplicateKey {
        side: &'static str,
        column: String,
        key: String,
    },
    DuplicateColumn {
        side: &'static str,
        column: String,
    },
    InvalidRow {
        side: &'static str,
        key: String,
    },
}

impl fmt::Display for DiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKeyColumn(key) => {
                write!(f, "key column '{key}' was not found in both CSV files")
            }
            Self::DuplicateKey { side, column, key } => {
                write!(
                    f,
                    "duplicate key '{key}' in {side} CSV file (column '{column}')"
                )
            }
            Self::DuplicateColumn { side, column } => {
                write!(f, "duplicate column '{column}' in {side} CSV header")
            }
            Self::InvalidRow { side, key } => {
                write!(f, "invalid row for key '{key}' in {side} CSV file")
            }
        }
    }
}

impl std::error::Error for DiffError {}

pub fn compare(
    old_headers: &[String],
    old_rows: &[Vec<String>],
    new_headers: &[String],
    new_rows: &[Vec<String>],
    key_column: &str,
) -> Result<DiffReport, DiffError> {
    let old_indexes = header_indexes(old_headers, "old")?;
    let new_indexes = header_indexes(new_headers, "new")?;
    let old_key_index = *old_indexes
        .get(key_column)
        .ok_or_else(|| DiffError::MissingKeyColumn(key_column.to_owned()))?;
    let new_key_index = *new_indexes
        .get(key_column)
        .ok_or_else(|| DiffError::MissingKeyColumn(key_column.to_owned()))?;

    let old_by_key = index_rows(old_rows, old_headers, old_key_index, key_column, "old")?;
    let new_by_key = index_rows(new_rows, new_headers, new_key_index, key_column, "new")?;

    let old_header_set: HashSet<&str> = old_headers.iter().map(String::as_str).collect();
    let new_header_set: HashSet<&str> = new_headers.iter().map(String::as_str).collect();
    let columns_added: Vec<String> = new_headers
        .iter()
        .filter(|column| !old_header_set.contains(column.as_str()))
        .cloned()
        .collect();
    let columns_removed: Vec<String> = old_headers
        .iter()
        .filter(|column| !new_header_set.contains(column.as_str()))
        .cloned()
        .collect();

    let mut added = Vec::new();
    for row in new_rows {
        let key = &row[new_key_index];
        if !old_by_key.contains_key(key.as_str()) {
            added.push(make_row(key, row, new_headers));
        }
    }

    let mut removed = Vec::new();
    let mut changed = Vec::new();
    for row in old_rows {
        let key = &row[old_key_index];
        let Some(new_row) = new_by_key.get(key.as_str()) else {
            removed.push(make_row(key, row, old_headers));
            continue;
        };
        for column in old_headers {
            let Some(&new_index) = new_indexes.get(column) else {
                continue;
            };
            let old_value = &row[old_indexes[column]];
            let new_value = &new_row[new_index];
            if old_value != new_value {
                changed.push(CellChange {
                    key: key.clone(),
                    column: column.clone(),
                    old: old_value.clone(),
                    new: new_value.clone(),
                });
            }
        }
    }

    let changed_rows = changed
        .iter()
        .map(|change| change.key.as_str())
        .collect::<HashSet<_>>()
        .len();
    let row_word = if changed_rows == 1 { "row" } else { "rows" };
    let mut summary = format!(
        "{} added, {} removed, {} cells changed in {} {}",
        added.len(),
        removed.len(),
        changed.len(),
        changed_rows,
        row_word
    );
    if !columns_added.is_empty() || !columns_removed.is_empty() {
        use fmt::Write;
        write!(
            summary,
            " ({} columns added, {} removed)",
            columns_added.len(),
            columns_removed.len()
        )
        .unwrap();
    }

    Ok(DiffReport {
        added,
        removed,
        changed,
        columns_added,
        columns_removed,
        summary,
    })
}

fn header_indexes(
    headers: &[String],
    side: &'static str,
) -> Result<HashMap<String, usize>, DiffError> {
    let mut indexes = HashMap::new();
    for (index, column) in headers.iter().enumerate() {
        if indexes.insert(column.clone(), index).is_some() {
            return Err(DiffError::DuplicateColumn {
                side,
                column: column.clone(),
            });
        }
    }
    Ok(indexes)
}

fn index_rows<'a>(
    rows: &'a [Vec<String>],
    headers: &[String],
    key_index: usize,
    key_column: &str,
    side: &'static str,
) -> Result<HashMap<&'a str, &'a Vec<String>>, DiffError> {
    let mut indexed = HashMap::new();
    for row in rows {
        if row.len() != headers.len() {
            return Err(DiffError::InvalidRow {
                side,
                key: row.get(key_index).cloned().unwrap_or_default(),
            });
        }
        let key = &row[key_index];
        if indexed.insert(key.as_str(), row).is_some() {
            return Err(DiffError::DuplicateKey {
                side,
                column: key_column.to_owned(),
                key: key.clone(),
            });
        }
    }
    Ok(indexed)
}

fn make_row(key: &str, row: &[String], headers: &[String]) -> Row {
    Row {
        key: key.to_owned(),
        values: headers
            .iter()
            .zip(row)
            .map(|(header, value)| (header.clone(), value.clone()))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn matches_columns_by_name_and_reports_row_changes() {
        let report = compare(
            &strings(&["id", "name", "quantity"]),
            &[
                strings(&["1", "Widget", "10"]),
                strings(&["2", "Gadget", "5"]),
            ],
            &strings(&["quantity", "id", "name"]),
            &[
                strings(&["14", "1", "Widget Pro"]),
                strings(&["8", "3", "Washer"]),
            ],
            "id",
        )
        .unwrap();
        assert_eq!(report.changed.len(), 2);
        assert_eq!(report.added.len(), 1);
        assert_eq!(report.removed.len(), 1);
        assert_eq!(
            report.summary,
            "1 added, 1 removed, 2 cells changed in 1 row"
        );
    }

    #[test]
    fn reports_columns_without_comparing_them_as_cell_changes() {
        let report = compare(
            &strings(&["id", "name"]),
            &[strings(&["1", "Widget"])],
            &strings(&["id", "name", "price"]),
            &[strings(&["1", "Widget", "3.00"])],
            "id",
        )
        .unwrap();
        assert_eq!(report.columns_added, strings(&["price"]));
        assert!(report.changed.is_empty());
    }

    #[test]
    fn reports_added_and_removed_columns() {
        let report = compare(
            &strings(&["id", "retired"]),
            &[strings(&["1", "old"])],
            &strings(&["id", "fresh"]),
            &[strings(&["1", "new"])],
            "id",
        )
        .unwrap();
        assert_eq!(report.columns_added, strings(&["fresh"]));
        assert_eq!(report.columns_removed, strings(&["retired"]));
        assert!(report.changed.is_empty());
        assert!(report.has_changes());
    }

    #[test]
    fn rejects_duplicate_keys() {
        let error = compare(
            &strings(&["id"]),
            &[strings(&["1"]), strings(&["1"])],
            &strings(&["id"]),
            &[strings(&["1"])],
            "id",
        )
        .unwrap_err();
        assert!(error.to_string().contains("duplicate key '1'"));
    }
}
