use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use chrono::prelude::*;
use xml::writer::events::StartElementBuilder;
use xml::writer::{EmitterConfig, EventWriter, Result, XmlEvent};

use crate::client::work_query::PublicationType;
use crate::client::work_query::SubjectType;
use crate::client::work_query::WorkQueryWork;
use crate::errors;

pub fn generate_onix_3(mut work: WorkQueryWork) -> errors::Result<()> {
    println!("{:#?}", work);

    let mut file = File::create("output.xml").unwrap();
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut file);
    match handle_event(&mut writer, &mut work) {
        Ok(_) => Ok(()),
        Err(e) => Err(errors::ThothError::from(e).into()),
    }
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn stype_to_scheme(subject_type: &SubjectType) -> &str {
    match subject_type {
        SubjectType::BIC => "12",
        SubjectType::BISAC => "10",
        SubjectType::KEYWORD => "20",
        SubjectType::LCC => "04",
        SubjectType::THEMA => "93",
        SubjectType::CUSTOM => "B2", // B2 Keywords (not for display)
        _ => unreachable!(),
    }
}

fn write_element_block<W: Write, F: Fn(&mut EventWriter<W>)>(
    element: &str,
    ns: Option<HashMap<String, String>>,
    attr: Option<HashMap<String, String>>,
    w: &mut EventWriter<W>,
    f: F,
) -> Result<()> {
    let mut event_builder: StartElementBuilder = XmlEvent::start_element(element);

    if let Some(ns) = ns {
        for (k, v) in ns.iter() {
            event_builder = event_builder.ns(
                string_to_static_str(k.clone()),
                string_to_static_str(v.clone()),
            );
        }
    }

    if let Some(attr) = attr {
        for (k, v) in attr.iter() {
            event_builder = event_builder.attr(
                string_to_static_str(k.clone()),
                string_to_static_str(v.clone()),
            );
        }
    }

    let mut event: XmlEvent = event_builder.into();
    w.write(event)?;
    f(w);
    event = XmlEvent::end_element().into();
    w.write(event)
}

fn handle_event<W: Write>(w: &mut EventWriter<W>, work: &mut WorkQueryWork) -> Result<()> {
    let ns_map: HashMap<String, String> = HashMap::new();
    let mut attr_map: HashMap<String, String> = HashMap::new();

    attr_map.insert(
        "xmlns".to_string(),
        "http://ns.editeur.org/onix/3.0/reference".to_string(),
    );
    attr_map.insert("release".to_string(), "3.0".to_string());

    let work_id = &work.work_id.to_string();
    let doi = match &work.doi.as_ref() {
        Some(doi) => doi.replace("https://doi.org/", ""),
        None => "".to_string(),
    };
    let page_count = match &work.page_count.as_ref() {
        Some(page_count) => page_count.to_string(),
        None => "".to_string(),
    };
    let subtitle = &work.subtitle.as_ref().unwrap();
    let mut isbn = "".to_string();
    for publication in &work.publications {
        if publication.publication_type.eq(&PublicationType::PDF) {
            isbn = match &publication.isbn.as_ref() {
                Some(isbn) => isbn.replace("-", ""),
                None => "".to_string(),
            };
            break;
        }
    }
    let license = match &work.license.as_ref() {
        Some(license) => (*license).to_string(),
        None => "".to_string(),
    };

    write_element_block("ONIXMessage", Some(ns_map), Some(attr_map), w, |w| {
        write_element_block("Header", None, None, w, |w| {
            write_element_block("Sender", None, None, w, |w| {
                write_element_block("SenderName", None, None, w, |w| {
                    let event: XmlEvent =
                        XmlEvent::Characters(&work.imprint.publisher.publisher_name);
                    w.write(event).ok();
                })
                .ok();
                write_element_block("EmailAddress", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("javi@openbookpublishers.com");
                    w.write(event).ok();
                })
                .ok();
            })
            .ok();
            write_element_block("SentDateTime", None, None, w, |w| {
                let utc = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
                let event: XmlEvent = XmlEvent::Characters(&utc);
                w.write(event).ok();
            })
            .ok();
        })
        .ok();

        write_element_block("Product", None, None, w, |w| {
            write_element_block("RecordReference", None, None, w, |w| {
                let event: XmlEvent = XmlEvent::Characters(&work_id);
                w.write(event).ok();
            })
            .ok();
            // 03 Notification confirmed on publication
            write_element_block("NotificationType", None, None, w, |w| {
                let event: XmlEvent = XmlEvent::Characters("03");
                w.write(event).ok();
            })
            .ok();
            // 01 Publisher
            write_element_block("RecordSourceType", None, None, w, |w| {
                let event: XmlEvent = XmlEvent::Characters("01");
                w.write(event).ok();
            })
            .ok();
            write_element_block("ProductIdentifier", None, None, w, |w| {
                // 01 Proprietary
                write_element_block("ProductIDType", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("01");
                    w.write(event).ok();
                })
                .ok();
                write_element_block("IDValue", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters(work_id);
                    w.write(event).ok();
                })
                .ok();
            })
            .ok();
            write_element_block("ProductIdentifier", None, None, w, |w| {
                // 15 ISBN-13
                write_element_block("ProductIDType", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("15");
                    w.write(event).ok();
                })
                .ok();
                write_element_block("IDValue", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters(&isbn);
                    w.write(event).ok();
                })
                .ok();
            })
            .ok();
            if !doi.is_empty() {
                write_element_block("ProductIdentifier", None, None, w, |w| {
                    write_element_block("ProductIDType", None, None, w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("06");
                        w.write(event).ok();
                    })
                    .ok();
                    write_element_block("IDValue", None, None, w, |w| {
                        let event: XmlEvent = XmlEvent::Characters(&doi);
                        w.write(event).ok();
                    })
                    .ok();
                })
                .ok();
            }
            write_element_block("DescriptiveDetail", None, None, w, |w| {
                // 00 Single-component retail product
                write_element_block("ProductComposition", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("00");
                    w.write(event).ok();
                })
                .ok();
                // EB Digital download and online
                write_element_block("ProductForm", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("EB");
                    w.write(event).ok();
                })
                .ok();
                // E107 PDF
                write_element_block("ProductFormDetail", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("E107");
                    w.write(event).ok();
                })
                .ok();
                // 10 Text (eye-readable)
                write_element_block("PrimaryContentType", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("10");
                    w.write(event).ok();
                })
                .ok();
                if !license.is_empty() {
                    write_element_block("EpubLicense", None, None, w, |w| {
                        write_element_block("EpubLicenseName", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("Creative Commons License");
                            w.write(event).ok();
                        })
                        .ok();
                        write_element_block("EpubLicenseExpression", None, None, w, |w| {
                            write_element_block("EpubLicenseExpressionType", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("02");
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("EpubLicenseExpressionLink", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(&license);
                                w.write(event).ok();
                            })
                            .ok();
                        })
                        .ok();
                    })
                    .ok();
                }
                write_element_block("TitleDetail", None, None, w, |w| {
                    // 01 Distinctive title (book)
                    write_element_block("TitleType", None, None, w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("01");
                        w.write(event).ok();
                    })
                    .ok();
                    write_element_block("TitleElement", None, None, w, |w| {
                        // 01 Product
                        write_element_block("TitleElementLevel", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("01");
                            w.write(event).ok();
                        })
                        .ok();
                        if subtitle.is_empty() {
                            write_element_block("TitleText", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(&work.full_title);
                                w.write(event).ok();
                            })
                            .ok();
                        } else {
                            write_element_block("TitleText", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(&work.title);
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("Subtitle", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(subtitle);
                                w.write(event).ok();
                            })
                            .ok();
                        }
                    })
                    .ok();
                })
                .ok();
                if !page_count.is_empty() {
                    write_element_block("Extent", None, None, w, |w| {
                        // 00 Main content
                        write_element_block("ExtentType", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("00");
                            w.write(event).ok();
                        })
                        .ok();
                        write_element_block("ExtentValue", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters(&page_count);
                            w.write(event).ok();
                        })
                        .ok();
                        // 03 Pages
                        write_element_block("ExtentUnit", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("03");
                            w.write(event).ok();
                        })
                        .ok();
                    })
                    .ok();
                }
                for subject in &work.subjects {
                    write_element_block("Subject", None, None, w, |w| {
                        // 00 Main content
                        write_element_block("SubjectSchemeIdentifier", None, None, w, |w| {
                            let scheme = stype_to_scheme(&subject.subject_type);
                            let event: XmlEvent = XmlEvent::Characters(scheme);
                            w.write(event).ok();
                        })
                        .ok();
                        write_element_block("SubjectCode", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters(&subject.subject_code);
                            w.write(event).ok();
                        })
                        .ok();
                    })
                    .ok();
                }
            })
            .ok();
        })
        .ok();
    })
}