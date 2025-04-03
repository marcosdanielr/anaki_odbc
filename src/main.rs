use anyhow::Error;
use odbc_api::{ConnectionOptions, Cursor, Environment, ResultSetMetadata, buffers::TextRowSet};

use std::time::Instant;

const BATCH_SIZE: usize = 5_000;

fn main() -> Result<(), Error> {
    let environment = Environment::new()?;

    let connection = environment.connect_with_connection_string(
        "DSN=PostgreSQL;UID=user;PWD=password;",
        ConnectionOptions::default(),
    )?;

    let start_time = Instant::now();

    match connection.execute("SELECT * FROM test_table", (), None)? {
        Some(mut cursor) => {
            let _headline: Vec<String> = cursor.column_names()?.collect::<Result<_, _>>()?;

            let mut buffers = TextRowSet::for_cursor(BATCH_SIZE, &mut cursor, Some(4096))?;
            let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;

            while let Some(batch) = row_set_cursor.fetch()? {
                for _row_index in 0..batch.num_rows() {
                    // let _record: Vec<String> = (0..batch.num_cols())
                    //     .map(|col_index| {
                    //         let value = batch.at(col_index, row_index).unwrap_or(&[]);
                    //         String::from_utf8_lossy(value).to_string()
                    //     })
                    //     .collect();

                    // println!("{}", record.join(","));
                }
            }
        }
        None => {
            eprintln!("Query came back empty. No output has been created.");
        }
    }

    let duration = start_time.elapsed();

    println!("Tempo total: {:.9}", duration.as_secs_f64());

    Ok(())
}
