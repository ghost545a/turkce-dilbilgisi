//! Turkish-aware dictionary lookup (ported from the desktop app's
//! `dictionary.rs`). A `HashMap` built from the TSV replaces SQLite; the order
//! in which candidate forms are tried is unchanged.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::Serialize;

/// Strips surrounding punctuation while keeping internal apostrophes
/// (`"Ayşe'nin,"` -> `"Ayşe'nin"`, `"«Merhaba!»"` -> `"Merhaba"`).
pub fn clean_word(raw: &str) -> String {
    raw.trim_matches(|c: char| !c.is_alphabetic()).to_string()
}

/// Lowercases with Turkish dotted / dotless I rules.
pub fn turkish_lowercase(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'İ' => out.push('i'),
            'I' => out.push('ı'),
            _ => out.extend(c.to_lowercase()),
        }
    }
    out
}

/// Titlecases a Turkish lowercase string (`'i' -> 'İ'`, `'ı' -> 'I'`).
pub fn turkish_titlecase(s: &str) -> String {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = String::with_capacity(s.len() + 2);
    match first {
        'i' => out.push('İ'),
        'ı' => out.push('I'),
        _ => out.extend(first.to_uppercase()),
    }
    out.push_str(chars.as_str());
    out
}

/// `kitab` -> `kitap`, `gid` -> `git`.
fn unsoften_final_consonant(stem: &str) -> Option<String> {
    let mut chars: Vec<char> = stem.chars().collect();
    let last = chars.last_mut()?;
    *last = match *last {
        'b' => 'p',
        'c' => 'ç',
        'd' => 't',
        'ğ' => 'k',
        _ => return None,
    };
    Some(chars.into_iter().collect())
}

/// `A` = e/a, `I` = i/ı/u/ü, `D` = d/t, `C` = c/ç.
const SUFFIX_TEMPLATES: &[&str] = &[
    "lAr", "lArI", "lArIn", "I", "yI", "nI", "In", "nIn", "A", "yA", "nA", "DA", "nDA", "DAn",
    "nDAn", "lA", "ylA", "Im", "ImIz", "InIz", "sI", "sInI", "m", "n", "mIz", "nIz", "ki", "lI",
    "sIz", "lIk", "CI", "CA", "DIr", "yIm", "sIn", "Iz", "yIz", "sInIz", "yDI", "ymIş", "ysA",
    "Iyor", "yor", "DI", "DIm", "DIn", "DIk", "DInIz", "DIlAr", "mIş", "AcAk", "AcAğ", "yAcAk",
    "yAcAğ", "Ar", "Ir", "r", "mAz", "mA", "mAk", "mAlI", "Abil", "yAbil", "sA", "Ip", "yIp",
    "ArAk", "yArAk", "An", "yAn", "DIğ", "IncA", "yIncA", "ken", "yken", "Il", "In", "Iş",
];

fn expand_template(template: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    for c in template.chars() {
        let options: &[char] = match c {
            'A' => &['e', 'a'],
            'I' => &['i', 'ı', 'u', 'ü'],
            'D' => &['d', 't'],
            'C' => &['c', 'ç'],
            _ => &[],
        };
        if options.is_empty() {
            for s in &mut out {
                s.push(c);
            }
        } else {
            out = out.iter().flat_map(|s| options.iter().map(move |o| format!("{s}{o}"))).collect();
        }
    }
    out
}

fn all_suffixes() -> &'static [String] {
    static SUFFIXES: OnceLock<Vec<String>> = OnceLock::new();
    SUFFIXES.get_or_init(|| {
        let mut set: Vec<String> = SUFFIX_TEMPLATES.iter().flat_map(|t| expand_template(t)).collect();
        set.sort();
        set.dedup();
        set
    })
}

fn infinitive_suffix(stem: &str) -> &'static str {
    match stem.chars().rev().find(|c| "aeıioöuü".contains(*c)) {
        Some('e' | 'i' | 'ö' | 'ü') => "mek",
        _ => "mak",
    }
}

/// The stem, the unsoftened stem, their infinitives, and the infinitive with
/// a restored stem vowel (`bekl` + `iyor` -> `beklemek`).
fn stem_variants(stem: &str) -> Vec<String> {
    let mut out = vec![stem.to_string()];
    let unsoftened = unsoften_final_consonant(stem);
    if let Some(u) = &unsoftened {
        out.push(u.clone());
    }
    out.push(format!("{stem}{}", infinitive_suffix(stem)));
    if let Some(u) = &unsoftened {
        out.push(format!("{u}{}", infinitive_suffix(u)));
    }
    let restored = if infinitive_suffix(stem) == "mek" { 'e' } else { 'a' };
    out.push(format!("{stem}{restored}{}", infinitive_suffix(stem)));
    out
}

/// Peels up to three suffixes, nearest (longest stem) first.
fn candidate_stems(lower: &str) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(lower.to_string());
    let mut ordered = Vec::new();
    let mut frontier = vec![lower.to_string()];
    for _ in 0..3 {
        let mut next = Vec::new();
        for word in &frontier {
            for suffix in all_suffixes() {
                if let Some(stem) = word.strip_suffix(suffix.as_str()) {
                    if stem.chars().count() >= 2 && seen.insert(stem.to_string()) {
                        next.push(stem.to_string());
                    }
                }
            }
        }
        next.sort_by_key(|s| std::cmp::Reverse(s.chars().count()));
        ordered.extend(next.iter().cloned());
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }
    ordered
}

/// A successful lookup.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LookupResult {
    /// The cleaned word that was clicked.
    pub word: String,
    /// English meaning.
    pub translation: String,
    /// Grammar context, e.g. `"locative singular of ev"`; empty for headwords.
    pub note: String,
}

struct Row {
    translation: String,
    lemma: String,
    note: String,
}

/// The offline Turkish–English dictionary.
pub struct Dictionary {
    rows: HashMap<String, Row>,
}

impl Dictionary {
    /// Builds the dictionary from `word \t translation \t lemma \t note` lines.
    /// Later lines replace earlier ones, as `INSERT OR REPLACE` did.
    pub fn from_tsv(tsv: &str) -> Self {
        let mut rows = HashMap::with_capacity(40_000);
        for line in tsv.lines() {
            let mut cols = line.trim_end_matches('\r').split('\t');
            let (Some(word), Some(translation)) = (cols.next(), cols.next()) else {
                continue;
            };
            if word.is_empty() {
                continue;
            }
            rows.insert(
                word.to_string(),
                Row {
                    translation: translation.to_string(),
                    lemma: cols.next().unwrap_or("").to_string(),
                    note: cols.next().unwrap_or("").to_string(),
                },
            );
        }
        Self { rows }
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Exact, then lowercase, then Titlecase (proper nouns like `İstanbul`).
    fn row_any_case(&self, word: &str) -> Option<(String, &Row)> {
        let lower = turkish_lowercase(word);
        let title = turkish_titlecase(&lower);
        let mut tried: Vec<&str> = Vec::with_capacity(3);
        for candidate in [word, lower.as_str(), title.as_str()] {
            if candidate.is_empty() || tried.contains(&candidate) {
                continue;
            }
            tried.push(candidate);
            if let Some(row) = self.rows.get(candidate) {
                return Some((candidate.to_string(), row));
            }
        }
        None
    }

    /// Follows an inflected form to its headword when it has no meaning of its own.
    fn resolve(&self, word: &str, matched: &str, row: &Row, inflected_from: Option<&str>) -> Option<LookupResult> {
        let lemma_row = if row.lemma.is_empty() {
            None
        } else {
            self.row_any_case(&row.lemma).filter(|(_, r)| !r.translation.is_empty())
        };
        let form_label = |note: &str, lemma: &str| {
            if note.is_empty() { format!("form of {lemma}") } else { format!("{note} of {lemma}") }
        };
        let (translation, mut note) = match (row.translation.is_empty(), lemma_row) {
            (true, Some((_, lemma))) => (lemma.translation.clone(), form_label(&row.note, &row.lemma)),
            (true, None) => return None,
            (false, Some((_, lemma))) => (
                row.translation.clone(),
                format!("also {}: {}", form_label(&row.note, &row.lemma), lemma.translation),
            ),
            (false, None) => (row.translation.clone(), String::new()),
        };
        if let Some(surface) = inflected_from {
            if note.is_empty() && turkish_lowercase(matched) != turkish_lowercase(surface) {
                note = format!("form of {matched}");
            }
        }
        Some(LookupResult { word: word.to_string(), translation, note })
    }

    /// Finds the English meaning of a clicked subtitle word: exact or case-folded
    /// match, the part before an apostrophe, then stripped suffixes with
    /// unsoftening and infinitive reconstruction.
    pub fn lookup(&self, raw_word: &str) -> Option<LookupResult> {
        let cleaned = clean_word(raw_word);
        if cleaned.is_empty() {
            return None;
        }
        let lower = turkish_lowercase(&cleaned);

        if let Some((matched, row)) = self.row_any_case(&cleaned) {
            if let Some(found) = self.resolve(&cleaned, &matched, row, None) {
                return Some(found);
            }
        }

        let mut base = lower.clone();
        if let Some((stem, _)) = lower.split_once(['\'', '’']) {
            if !stem.is_empty() {
                if let Some((matched, row)) = self.row_any_case(stem) {
                    if let Some(found) = self.resolve(&cleaned, &matched, row, Some(&cleaned)) {
                        return Some(found);
                    }
                }
                base = stem.to_string();
            }
        }

        for stem in candidate_stems(&base) {
            for variant in stem_variants(&stem) {
                if let Some((matched, row)) = self.row_any_case(&variant) {
                    if let Some(found) = self.resolve(&cleaned, &matched, row, Some(&cleaned)) {
                        return Some(found);
                    }
                }
            }
        }
        None
    }

    /// `lookup` as JSON (`null` when nothing matches).
    pub fn lookup_json(&self, raw_word: &str) -> String {
        match self.lookup(raw_word) {
            Some(r) => serde_json::to_string(&r).unwrap_or_else(|_| "null".into()),
            None => "null".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dict() -> &'static Dictionary {
        static D: OnceLock<Dictionary> = OnceLock::new();
        D.get_or_init(|| Dictionary::from_tsv(include_str!("../../player/tr_en_dictionary.tsv")))
    }

    fn meaning(word: &str) -> String {
        dict()
            .lookup(word)
            .unwrap_or_else(|| panic!("'{word}' not found"))
            .translation
            .to_lowercase()
    }

    #[test]
    fn cleans_punctuation_and_turkish_case() {
        assert_eq!(clean_word("«Merhaba!»"), "Merhaba");
        assert_eq!(clean_word("ister?"), "ister");
        assert_eq!(clean_word("Ayşe'nin,"), "Ayşe'nin");
        assert_eq!(turkish_lowercase("İSTANBUL IŞIK"), "istanbul ışık");
        assert_eq!(turkish_titlecase("istanbul"), "İstanbul");
    }

    #[test]
    fn loads_the_full_dictionary() {
        assert!(dict().len() > 30_000, "only {} entries", dict().len());
    }

    #[test]
    fn everyday_words() {
        assert!(meaning("Merhaba,").contains("hello"));
        assert!(meaning("kahve").contains("coffee"));
        assert!(meaning("güzel").contains("beautiful"));
        assert!(meaning("teşekkür").contains("thank"));
        assert!(meaning("istanbul'da").contains("istanbul"));
    }

    #[test]
    fn inflected_words_resolve_to_their_headword() {
        assert!(meaning("evde").contains("house"));
        assert!(meaning("kitabı").contains("book"));
        assert!(meaning("arkadaşlarım").contains("friend"));
        assert!(meaning("evlerinde").contains("house"));
        assert!(meaning("gidiyorum").contains("go"));
        assert!(meaning("bekliyor").contains("wait"));
        assert!(meaning("söyledi").contains("say"));
        assert!(meaning("gelmiyor").contains("come"));
        assert!(meaning("ister").contains("want"));
        let evde = dict().lookup("evde").unwrap();
        assert!(evde.note.contains("ev"), "note was '{}'", evde.note);
        assert_eq!(dict().lookup("zzzqqqxxx"), None);
        assert_eq!(dict().lookup_json("zzzqqqxxx"), "null");
    }

    #[test]
    fn every_word_of_the_sample_lesson_is_found() {
        let tr = include_str!("../../player/samples/sample_tr.srt");
        let cues = super::super::srt::parse(tr).unwrap();
        for cue in cues {
            for word in cue.text.split_whitespace() {
                assert!(dict().lookup(word).is_some(), "'{word}' not found");
            }
        }
    }
}
