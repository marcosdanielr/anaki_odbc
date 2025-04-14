use odbc_api::{Cursor, Error, buffers::TextRowSet};
use rmp_serde::Serializer;
use serde::Serialize;

#[derive(Serialize)]
struct Header {
    columns: Vec<String>,
}

pub fn field_to_value(field: &[u8]) -> serde_json::Value {
    let field_str = String::from_utf8_lossy(field);

    if let Ok(num) = field_str.parse::<f64>() {
        if num.fract() == 0.0 {
            serde_json::json!(num as i64)
        } else {
            serde_json::json!(num)
        }
    } else {
        serde_json::json!(field_str)
    }
}

pub fn row_to_map(
    batch: &TextRowSet,
    row_index: usize,
    col_names: &[String],
) -> serde_json::Map<String, serde_json::Value> {
    let mut row = serde_json::Map::new();

    for (col_index, col_name) in col_names.iter().enumerate() {
        let value = batch.at(col_index, row_index).unwrap_or(&[]);
        row.insert(col_name.clone(), field_to_value(value));
    }

    row
}

pub fn column_names_to_header<C>(cursor: &mut C) -> Result<Vec<String>, Error>
where
    C: Cursor,
{
    cursor
        .column_names()?
        .map(|name| name.map(|s| s.to_string()))
        .collect::<Result<Vec<_>, _>>()
}

pub fn stream_header<C, F>(cursor: &mut C, mut on_binary: F) -> Result<Vec<String>, Error>
where
    C: Cursor,
    F: FnMut(Vec<u8>),
{
    let col_names = column_names_to_header(cursor)?;
    let header = Header {
        columns: col_names.clone(),
    };

    let mut buf = Vec::new();
    header.serialize(&mut Serializer::new(&mut buf)).unwrap();
    on_binary(buf);

    Ok(col_names)
}

pub fn stream_rows<C, F>(
    mut cursor: C,
    on_binary: &mut F,
    batch_size: usize,
    buffer_size: usize,
    col_names: Vec<String>,
) -> Result<(), Error>
where
    C: Cursor,
    F: FnMut(Vec<u8>),
{
    let mut buffers = TextRowSet::for_cursor(batch_size, &mut cursor, Some(buffer_size))?;
    let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;

    while let Some(batch) = row_set_cursor.fetch()? {
        for row_index in 0..batch.num_rows() {
            let row_map = row_to_map(&batch, row_index, &col_names);
            let mut buf = Vec::new();
            row_map.serialize(&mut Serializer::new(&mut buf)).unwrap();
            on_binary(buf);
        }
    }

    Ok(())
}
