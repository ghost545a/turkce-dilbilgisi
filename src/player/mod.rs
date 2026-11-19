//! Logic for the `/player/` page: a web port of the TRLearn desktop player's
//! SRT parser and Turkish-aware dictionary lookup.
//!
//! The dictionary data (`player/tr_en_dictionary.tsv`, from English Wiktionary
//! via kaikki.org, CC BY-SA 4.0) is fetched by the page and handed to
//! [`lookup::Dictionary::from_tsv`], so the map pages never download it.

pub mod lookup;
pub mod srt;
