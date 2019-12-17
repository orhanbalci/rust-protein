use super::entity::*;
use super::primitive::*;
use nom::{
    character::complete::{line_ending, space0, space1},
    do_parse, many0, map, named, opt,
};

use itertools::Itertools;

use std::{str, str::FromStr};

#[allow(dead_code)]
struct RevdatLine {
    modification_number: u32,
    continuation: u32,
    rest: String,
}

named!(
    revdat_line_parser<RevdatLine>,
    do_parse!(
        revdat
            >> space1
            >> modification_number: twodigit_integer
            >> cont: opt!(twodigit_integer)
            >> rest: till_line_ending
            >> line_ending
            >> (RevdatLine {
                modification_number,
                continuation: if let Some(cc) = cont { cc } else { 0 },
                rest: String::from_str(str::from_utf8(rest).unwrap()).unwrap(),
            })
    )
);

named!(
    revdat_line_folder<Vec<RevdatLine>>,
    map!(many0!(revdat_line_parser), |revdat: Vec<RevdatLine>| {
        revdat
            .into_iter()
            .group_by(|a| a.modification_number)
            .into_iter()
            .map(|(k, v)| RevdatLine {
                modification_number: k,
                continuation: 0,
                rest: String::from_utf8(v.fold(Vec::new(), |accu: Vec<u8>, sr: RevdatLine| {
                    accu.into_iter().chain(sr.rest.into_bytes()).collect()
                }))
                .unwrap(),
            })
            .collect::<Vec<_>>()
    })
);

named!(
    revdat_record_parser<Vec<Record>>,
    map!(revdat_line_folder, |revdat: Vec<RevdatLine>| {
        revdat
            .into_iter()
            .map(|r: RevdatLine| {
                let input = r.rest.into_bytes();
                let single_modification_parser_result = revdat_inner_parser(input.as_slice());
                match single_modification_parser_result {
                    Ok((_, mut single_revdat_record)) => {
                        match single_revdat_record {
                            Record::Revdat {
                                ref mut modification_number,
                                ..
                            } => {
                                *modification_number = r.modification_number;
                            }
                            _ => (),
                        }
                        single_revdat_record
                    }
                    _ => Record::Revdat {
                        modification_number: 0,
                        modification_date: chrono::naive::MIN_DATE,
                        idcode: String::new(),
                        modification_type: ModificationType::InitialRelease,
                        modification_detail: Vec::new(),
                    },
                }
            })
            .collect()
    })
);

named!(
    revdat_inner_parser<Record>,
    do_parse!(
        space0
            >> modification_date: date_parser
            >> space1
            >> idcode: alphanum_word
            >> space1
            >> modification_type: modification_type_parser
            >> space1
            >> modification_detail: idcode_list
            >> (Record::Revdat {
                modification_number: 0,
                modification_date,
                idcode,
                modification_type,
                modification_detail,
            })
    )
);