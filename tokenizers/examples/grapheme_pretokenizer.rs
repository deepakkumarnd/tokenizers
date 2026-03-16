use tokenizers::pre_tokenizers::grapheme::Grapheme;
use tokenizers::{OffsetReferential, OffsetType, PreTokenizedString, PreTokenizer};

fn main() {
    let pretok = Grapheme;

    let inputs = [
        "Hello",
        "café",           // multibyte codepoint: é is 2 bytes
        "hi👋🏽!",         // emoji + skin-tone modifier = 1 grapheme cluster
        "e\u{0301}xact",  // combining acute accent — e + ◌́ = 1 grapheme cluster
        "日本語",
    ];

    for input in &inputs {
        let mut pretokenized = PreTokenizedString::from(*input);
        pretok.pre_tokenize(&mut pretokenized).unwrap();

        let splits: Vec<_> = pretokenized
            .get_splits(OffsetReferential::Original, OffsetType::Byte)
            .into_iter()
            .map(|(s, offsets, _)| (s.to_owned(), offsets))
            .collect();

        println!("Input : {:?}", input);
        println!("Splits: {splits:?}");
        println!();
    }
}
