use unicode_segmentation::UnicodeSegmentation;

use crate::normalizer::Range;
use crate::tokenizer::{PreTokenizedString, PreTokenizer, Result};
use crate::utils::macro_rules_attribute;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Pre-tokenizer that splits text into individual Unicode grapheme clusters.
///
/// This correctly handles multi-codepoint characters such as emoji with skin tones,
/// combining diacritics, and other complex sequences, treating each grapheme cluster
/// as a single token.
#[macro_rules_attribute(impl_serde_type!)]
pub struct Grapheme;

impl Default for Grapheme {
    fn default() -> Self {
        Self
    }
}

impl PreTokenizer for Grapheme {
    fn pre_tokenize(&self, pretokenized: &mut PreTokenizedString) -> Result<()> {
        pretokenized.split(|_, normalized| {
            let text = normalized.get();
            if text.is_empty() {
                return Ok(vec![]);
            }

            let mut splits = Vec::new();
            for (start, grapheme) in text.grapheme_indices(true) {
                let end = start + grapheme.len();
                splits.push(
                    normalized
                        .slice(Range::Normalized(start..end))
                        .ok_or("Failed to slice normalized text")?,
                );
            }

            Ok(splits)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OffsetReferential, OffsetType, PreTokenizer};

    fn get_splits(pretok: &Grapheme, s: &str) -> Vec<(String, (usize, usize))> {
        let mut pretokenized = PreTokenizedString::from(s);
        pretok.pre_tokenize(&mut pretokenized).unwrap();
        pretokenized
            .get_splits(OffsetReferential::Original, OffsetType::Byte)
            .into_iter()
            .map(|(s, o, _)| (s.to_owned(), o))
            .collect()
    }

    #[test]
    fn basic_ascii() {
        let pretok = Grapheme;
        assert_eq!(
            get_splits(&pretok, "Hello"),
            vec![
                ("H".into(), (0, 1)),
                ("e".into(), (1, 2)),
                ("l".into(), (2, 3)),
                ("l".into(), (3, 4)),
                ("o".into(), (4, 5)),
            ]
        );
    }

    #[test]
    fn multibyte_chars() {
        // Each of these is a single codepoint but multi-byte in UTF-8
        let pretok = Grapheme;
        // "é" U+00E9 is 2 bytes in UTF-8
        assert_eq!(
            get_splits(&pretok, "café"),
            vec![
                ("c".into(), (0, 1)),
                ("a".into(), (1, 2)),
                ("f".into(), (2, 3)),
                ("é".into(), (3, 5)),
            ]
        );
    }

    #[test]
    fn emoji_with_skin_tone() {
        // 👋🏽 is a grapheme cluster: U+1F44B + U+1F3FD (wave + medium skin tone modifier)
        let pretok = Grapheme;
        let input = "hi👋🏽!";
        let splits = get_splits(&pretok, input);
        assert_eq!(splits[0], ("h".into(), (0, 1)));
        assert_eq!(splits[1], ("i".into(), (1, 2)));
        // The emoji cluster occupies bytes 2..10
        assert_eq!(splits[2].0, "👋🏽");
        assert_eq!(splits[3], ("!".into(), (splits[3].1.0, splits[3].1.1)));
    }

    #[test]
    fn combining_diacritics() {
        // "e\u{0301}" is 'e' + combining acute accent — one grapheme, two codepoints
        let pretok = Grapheme;
        let input = "e\u{0301}x";
        let splits = get_splits(&pretok, input);
        // First grapheme cluster: "e\u{0301}" (3 bytes: 1 + 2)
        assert_eq!(splits[0].0, "e\u{0301}");
        assert_eq!(splits[0].1, (0, 3));
        assert_eq!(splits[1], ("x".into(), (3, 4)));
    }

    #[test]
    fn empty_string() {
        let pretok = Grapheme;
        assert_eq!(get_splits(&pretok, ""), vec![]);
    }
}
