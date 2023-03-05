#![allow(dead_code)]

use quick_xml::{
    events::{BytesText, Event, BytesStart},
    Reader, Writer,
};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{self, Cursor},
    path::PathBuf,
};

fn main() -> Result<(), Box<dyn Error>> {
    //let files = get_files_paths()?;
    //println!("TOTAL FILES: {}", files.len());

    let changes = HashMap::from([("1", "1022"), ("2", "1022")]);

    let mut reader = Reader::from_file("example.xml")?;
    let mut buf = Vec::new();

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut new_cest: Option<&&str> = None;
    let mut should_skip_cest = false;

    loop {
        buf.clear();

        match reader.read_event_into(&mut buf) {
            Ok(e) => match &e {
                Event::Start(tag) if tag.name().as_ref() == b"prod" => {
                    writer.write_event(&e)?;

                    for att in tag
                        .attributes()
                        .with_checks(false)
                        .filter_map(|att| att.ok())
                    {
                        if att.key.as_ref() != b"nItem" {
                            continue;
                        }

                        let cest = changes.get(std::str::from_utf8(att.value.as_ref())?);

                        new_cest = cest;
                    }
                }
                Event::Start(tag) if tag.name().as_ref() == b"CEST" => {
                    writer.write_event(&e)?;

                    if new_cest.is_some() {
                        should_skip_cest = true;

                        let cest_content = new_cest.unwrap();

                        let cest_content_event =
                            Event::Text(BytesText::from_escaped(*cest_content));

                        writer.write_event(cest_content_event)?;
                    }
                }
                Event::Text(_) => {
                    if should_skip_cest {
                        should_skip_cest = false;
                    } else {
                        writer.write_event(&e)?;
                    }
                }
                Event::Eof => {
                    writer.write_event(&e)?;

                    break;
                }
                _ => writer.write_event(&e)?,
            },
            _ => (),
        }
    }

    Ok(())
}

fn get_files_paths() -> io::Result<Vec<PathBuf>> {
    let files: Vec<_> = fs::read_dir("/home/siws/.gnre/nfes-teste")?
        .into_iter()
        .filter_map(|entry| {
            entry.ok().and_then(|entry| match entry.path().extension() {
                Some(ext) if ext == "xml" => Some(entry.path()),
                _ => None,
            })
        })
        .collect();

    Ok(files)
}
