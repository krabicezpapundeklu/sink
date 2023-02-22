use std::{
    path::Path,
    str::{from_utf8, FromStr},
};

use anyhow::{anyhow, Context, Error, Result};
use csv::{ByteRecord, Reader};
use fancy_regex::Regex;
use lazy_static::lazy_static;
use log::info;
use rusqlite::Connection;

use crate::{
    repository::Repository,
    shared::{DateTime, Item, ItemHeader},
};

lazy_static! {
    static ref HEADER_REGEX: Regex = Regex::new(r"([\w_-]+)=(.*?)(?:}|, (?=[\w_-]+=))").unwrap();
}

pub fn import_csv(input: &Path, output: &Path, generate_ids: bool) -> Result<()> {
    let mut reader =
        Reader::from_path(input).with_context(|| format!("cannot read {}", input.display()))?;

    let mut db =
        Connection::open(output).with_context(|| format!("cannot open {}", output.display()))?;

    db.prepare_schema()
        .with_context(|| format!("cannot prepare database schema in {}", output.display()))?;

    let tx = db.transaction()?;

    let mut record = ByteRecord::new();
    let mut count = 0;

    while reader.read_byte_record(&mut record)? {
        let mut item = item_from_byte_record(&record).with_context(|| {
            record
                .position()
                .map_or("at unknown line".to_string(), |position| {
                    format!("at line {}", position.line())
                })
        })?;

        let id = item.id.unwrap();

        if generate_ids {
            item.id = None;
        }

        item.update_metadata();

        tx.insert_item_no_tx(&item)
            .with_context(|| format!("when inserting item {}", id))?;

        count += 1;

        if count % 1000 == 0 {
            info!("imported {count} items");
        }
    }

    if count % 1000 > 0 {
        info!("imported {count} items");
    }

    info!("commiting changes");

    tx.commit()?;

    Ok(())
}

fn item_from_byte_record(record: &ByteRecord) -> Result<Item> {
    let id = record
        .get(0)
        .ok_or_else(missing_column)
        .and_then(|id| from_utf8(id).map_err(Into::into))
        .and_then(|id| i64::from_str(id).map_err(Into::into))
        .context("cannot read ID")?;

    item_from_byte_record_with_id(id, record).with_context(|| format!("in item {id}"))
}

fn item_from_byte_record_with_id(id: i64, record: &ByteRecord) -> Result<Item> {
    let submit_date = record
        .get(1)
        .ok_or_else(missing_column)
        .and_then(|submit_date| from_utf8(submit_date).map_err(Into::into))
        .and_then(|submit_date| DateTime::from_str(submit_date).map_err(Into::into))
        .context("cannot read submit date")?;

    let body = record
        .get(2)
        .ok_or_else(missing_column)
        .context("cannot read body")?;

    let headers = record
        .get(3)
        .ok_or_else(missing_column)
        .and_then(|headers| from_utf8(headers).map_err(Into::into))
        .context("cannot read headers")?;

    let mut item = Item {
        id: Some(id),
        submit_date,
        system: None,
        r#type: None,
        headers: Vec::new(),
        body: body.to_vec(),
    };

    for header in HEADER_REGEX.captures_iter(headers) {
        let header = header?;

        let name = header
            .get(1)
            .ok_or_else(|| anyhow!("header without name"))?
            .as_str();

        let value = header
            .get(2)
            .ok_or_else(|| anyhow!("header without value"))?
            .as_str()
            .as_bytes();

        item.headers.push(ItemHeader {
            name: name.to_string(),
            value: value.to_vec(),
        });
    }

    Ok(item)
}

fn missing_column() -> Error {
    anyhow!("column doesn't exist")
}
