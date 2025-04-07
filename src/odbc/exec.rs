use odbc_api::{Connection, Error};

use super::util;

const BATCH_SIZE: usize = 5_000;
const BUFFER_SIZE: usize = 4_096;

pub fn execute<F>(conn: &mut Connection<'_>, sql: &str, mut on_csv: F) -> Result<(), Error>
where
    F: FnMut(String),
{
    let mut stmt = conn.prepare(sql)?;
    let executed = stmt.execute(())?;

    if let Some(mut cursor) = executed {
        util::stream_header(&mut cursor, &mut on_csv)?;
        util::stream_rows(cursor, &mut on_csv, BATCH_SIZE, BUFFER_SIZE)?;

        return Ok(());
    }

    drop(executed);

    let affected = stmt
        .row_count()
        .ok()
        .flatten()
        .map(|n| n.to_string())
        .unwrap_or_else(|| "unknown".into());

    on_csv(format!("__META__,affected_rows={}", affected));

    on_csv(format!("__META__,affected_rows={:?}", affected));

    Ok(())
}
