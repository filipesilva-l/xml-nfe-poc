#![allow(dead_code)]

use quick_xml::{
    events::{BytesText, Event},
    Reader, Writer,
};
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io,
    os::unix::prelude::MetadataExt,
    path::PathBuf,
};

const TAB_CODE: u8 = 9;
const INDENT_SIZE: usize = 1;

fn main() -> Result<(), Box<dyn Error>> {
    let changes = HashMap::from([("1", "1022")]);

    let mut reader = Reader::from_file("example.xml")?;

    let file = File::open("example.xml")?;
    let file_size: usize = file.metadata()?.size() as usize;
    drop(file);

    let mut buf = Vec::with_capacity(file_size);

    let result_file = File::create("result.xml")?;
    let mut writer = Writer::new(result_file);

    let mut new_cest: Option<&&str> = None;
    let mut should_change_text = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(e) => {
                let mut write_cest_text = || -> Result<(), Box<dyn Error>> {
                        let content = *new_cest.unwrap();
                        let content = BytesText::from_escaped(content);

                        let event = Event::Text(content);

                        writer.write_event(event)?;

                        Ok(())
                };

                match &e {
                    // change prod
                    Event::Start(tag) if tag.name().as_ref() == b"prod" => {
                        writer.write_event(&e)?;

                        for att in tag
                            .attributes()
                            .with_checks(false)
                            .filter_map(|att| att.ok())
                        {
                            if att.key.as_ref() != b"cod" {
                                continue;
                            }

                            new_cest = changes.get(std::str::from_utf8(att.value.as_ref())?);
                        }
                    }

                    // ignore CEST already existent
                    Event::Start(tag) if tag.name().as_ref() == b"cest" => {
                        should_change_text = new_cest.is_some();

                        writer.write_event(&e)?;
                    }

                    // change existing cest
                    Event::Text(_) if should_change_text => {
                        write_cest_text()?;

                        new_cest = None;
                        should_change_text = false;
                    },

                    // if we reached the end of the cest tag and should_change_text is still true,
                    // this means that the cast tag had no text inside, so we first write the new cest
                    // and then we end the tag
                    Event::End(tag) if tag.name().as_ref() == b"cest" && should_change_text => {
                        write_cest_text()?;

                        new_cest = None;
                        should_change_text = false;

                        writer.write_event(&e)?;
                    }

                    // ignore empty CEST tag
                    Event::Empty(tag) if tag.name().as_ref() == b"cest" => (),

                    // break from loop
                    Event::Eof => {
                        writer.write_event(&e)?;
                        break;
                    }

                    // write every other non-matched event to the new file
                    _ => writer.write_event(&e)?,
                }
            }
            _ => (),
        }
    }

    println!("{}", fs::read_to_string("result.xml")?);

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

