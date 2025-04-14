use odbc_api::{Connection, Error};
use rmp_serde::Serializer;
use serde::Serialize;

use super::util;

const BATCH_SIZE: usize = 5_000;
const BUFFER_SIZE: usize = 262_144;

#[derive(Serialize)]
struct Meta {
    affected_rows: String,
}

pub fn execute<F>(conn: &mut Connection<'_>, sql: &str, mut on_binary: F) -> Result<(), Error>
where
    F: FnMut(Vec<u8>),
{
    let mut stmt = conn.prepare(sql)?;
    let executed = stmt.execute(())?;

    if let Some(mut cursor) = executed {
        let col_names = util::stream_header(&mut cursor, &mut on_binary)?;
        util::stream_rows(cursor, &mut on_binary, BATCH_SIZE, BUFFER_SIZE, col_names)?;

        return Ok(());
    }

    drop(executed);

    let affected = stmt
        .row_count()
        .ok()
        .flatten()
        .map(|n| n.to_string())
        .unwrap_or_else(|| "unknown".into());

    let meta = Meta {
        affected_rows: affected,
    };
    let mut buf = Vec::new();
    meta.serialize(&mut Serializer::new(&mut buf)).unwrap();
    on_binary(buf);

    Ok(())
}
