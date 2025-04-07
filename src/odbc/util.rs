use odbc_api::{Cursor, Error, buffers::TextRowSet};

pub fn escape_csv_field(field: &[u8]) -> String {
    let mut field_str = String::from_utf8_lossy(field).to_string();

    if field_str.contains(',') || field_str.contains('"') {
        field_str = format!("\"{}\"", field_str.replace('"', "\"\""));
    }

    field_str
}

pub fn row_to_csv_line(batch: &TextRowSet, row_index: usize) -> String {
    let mut row = Vec::new();

    for col_index in 0..batch.num_cols() {
        let value = batch.at(col_index, row_index).unwrap_or(&[]);
        row.push(escape_csv_field(value));
    }

    row.join(",")
}

pub fn column_names_to_header<C>(cursor: &mut C) -> Result<String, Error>
where
    C: Cursor,
{
    let col_names = cursor
        .column_names()?
        .map(|name| name.map(|s| s.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(col_names.join(","))
}

pub fn stream_header<C, F>(cursor: &mut C, mut on_csv: F) -> Result<(), Error>
where
    C: Cursor,
    F: FnMut(String),
{
    let header = column_names_to_header(cursor)?;
    on_csv(header);
    Ok(())
}

pub fn stream_rows<C, F>(
    mut cursor: C,
    on_csv: &mut F,
    batch_size: usize,
    buffer_size: usize,
) -> Result<(), Error>
where
    C: Cursor,
    F: FnMut(String),
{
    let mut buffers = TextRowSet::for_cursor(batch_size, &mut cursor, Some(buffer_size))?;
    let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;

    while let Some(batch) = row_set_cursor.fetch()? {
        for row_index in 0..batch.num_rows() {
            on_csv(super::util::row_to_csv_line(&batch, row_index));
        }
    }

    Ok(())
}
