pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    use bstr::ByteSlice;
    use kitoken::Kitoken;

    let kt = Kitoken::from_tekken_file("tests/models/tekken/nemo.json")?;
    eprintln!("{:#?}", kt);
    let en = kt.encode("Unlock(", true)?;
    eprintln!("{:?}", en);
    let de = kt.decode(en, true)?;
    eprintln!("{:?}", de.as_bstr());

    // let kt = Kitoken::from_sentencepiece_file("tests/models/sentencepiece/llama2.model")?;
    // eprintln!("{:#?}", kt);
    // let en = kt.encode("hello world!", true)?;
    // eprintln!("{:?}", en);
    // let de = kt.decode(en, true)?;
    // eprintln!("{:?}", de.as_bstr());
    // kt.to_file("tests/models/llama2.kit")?;

    // let text = std::fs::read_to_string("tests/data/mixed_input.txt")?;

    // let encoded = kt.encode(&text, false)?;


    // use kitoken::Kitoken;
    // let kt = Kitoken::from_tokenizers_file("tests/models/tokenizers/vivaldai2.json")?;
    // eprintln!("{:#?}", kt);

    // // [28, 24, 28, 28, 28, 52, 4]
    // let e = kt.encode("sしeaレPe三et、 ڼi tޙ  っ時rB⟝wsの見3˪", true);
    // eprintln!("{:?}", e);
    // let d = kt.decode(e?, true);
    // eprintln!("{:?}", d?.as_bstr());

    // use kitoken::Kitoken;
    // let kt = Kitoken::from_sentencepiece_file("tests/models/sentencepiece/xlnet_base_cased.model")?;
    // eprintln!("{:#?}", kt);
    // let decoded = kt.decode([155, 0, 8273], true)?;
    // println!("{:?}", decoded.as_bstr());
    // let decoded = kt.decode([14070, 0, 93], true)?;
    // println!("{:?}", decoded.as_bstr());
    // let encoded = kt.encode("Զnnn", true)?;
    // println!("{:?}", encoded);
    // let encoded = kt.encode("ԶԶnnn", true)?;
    // println!("{:?}", encoded);
    // let encoded = kt.encode("nnn", true)?;
    // println!("{:?}", encoded);


    // println!("{:?}", kt.decode([96, 150, 2460, 0], true)?.as_bstr());
    // println!(
    //     "{:?}",
    //     [96, 150, 2460, 0].map(|w| kt.decode([w], true).unwrap().as_bstr().to_owned())
    // );
    // println!("{:?}", kt.decode([96, 780, 508, 0], true)?.as_bstr());
    // println!(
    //     "{:?}",
    //     [96, 780, 508, 0].map(|w| kt.decode([w], true).unwrap().as_bstr().to_owned())
    // );

    // println!("{:?}", kt.decode([0, 155, 202, 0], true)?.as_bstr());
    // println!(
    //     "{:?}",
    //     [0, 155, 202, 0].map(|w| kt.decode([w], true).unwrap().as_bstr().to_owned())
    // );

    // println!("{:?}", kt.decode([0, 155, 202, 0], true)?.as_bstr());
    // println!(
    //     "{:?}",
    //     [0, 8609, 23, 0].map(|w| kt.decode([w], true).unwrap().as_bstr().to_owned())
    // );


    eprintln!("{}", '🏃'.len_utf8());

    // use kitoken::{Kitoken, Regex, Split, SplitBehavior, TextPart};
    // let orig = Split::Pattern {
    //     pattern: Regex::new(
    //         r"'(?:[sdmt]|ll|ve|re)| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?!\S)|\s+",
    //     )?,

    //     behavior: SplitBehavior::Isolate,
    // };
    // let new = Split::Pattern {
    //     pattern:  Regex::new(
    //         r"'(?:[sdmt]|ll|ve|re)| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?![^\s])",
    //     )?,
    //     behavior: SplitBehavior::Isolate,
    // };

    // let text = "Double\n\n\nNewline\n　おりから門の格子《こうし》がチリン、チリン、チリリリリンと鳴る。大方来客であろう、来客なら下女が取次に出る";

    // let ids = orig.split(text);
    // let tokens = ids.iter().map(|t| &text[t.0..t.1]).collect::<Vec<_>>();
    // println!("o: {:?}", tokens);

    // let ids = new.split(text);
    // let tokens = ids.iter().map(|t| &text[t.0..t.1]).collect::<Vec<_>>();
    // println!("n: {:?}", tokens);

    // let k = Kitoken::from_sentencepiece_file("tests/models/sentencepiece/llama2.model")?;

    // let mut text = "<s>[PAD]</s>".into();
    // k.config.normalize(&mut text);
    // println!("{:?}", text);
    // let splits = k.config.split(text.as_ref());
    // println!("{:?}", splits.iter().map(|&(from, to)| &text[from..to]).collect::<Vec<_>>());
    // println!(
    //     "{:?}",
    //     [
    //         29871, 29966, 29879, 29958, 29961, 29925, 3035, 29962, 829, 29879, 29958
    //     ]
    //     .map(|s| k.decoder.get(&s).unwrap().as_bstr())
    // );
    // println!(
    //     "{:?}",
    //     [529, 29879, 24566, 29925, 3035, 29962, 829, 29879, 29958].map(|s| k
    //         .decoder
    //         .get(&s)
    //         .unwrap()
    //         .as_bstr())
    // );

    // let kt = Kitoken::from_sentencepiece_file("tests/models/sentencepiece/llama2.model")?;
    // let mut text = "<s>[PAD]</s>".into();
    // kt.config.normalize(&mut text);
    // println!("normed: {:?}", text);
    // let splits = kt.split_into_parts(&text, true);
    // println!(
    //     "split(true): {:?}",
    //     splits.iter().map(|&p| &text[p.range()]).collect::<Vec<_>>()
    // );
    // let splits = kt.split_into_parts(&text, false);
    // println!(
    //     "split(false): {:?}",
    //     splits.iter().map(|&p| &text[p.range()]).collect::<Vec<_>>()
    // );
    // println!("{:?}", kt.encode("<s>[PAD]</s>", true)?);

    // println!(
    //     "{:?}",
    //     [
    //         29871, 29966, 29879, 29958, 29961, 29925, 3035, 29962, 829, 29879, 29958
    //     ]
    //     .map(|s| k
    //         .special_decoder
    //         .get(&s)
    //         .map(|s| Some(&s.bytes))
    //         .unwrap_or_else(|| k.decoder.get(&s))
    //         .unwrap()
    //         .as_bstr())
    // );
    // println!(
    //     "{:?}",
    //     [529, 29879, 24566, 29925, 3035, 29962, 829, 29879, 29958].map(|s| k
    //         .special_decoder
    //         .get(&s)
    //         .map(|s| Some(&s.bytes))
    //         .unwrap_or_else(|| k.decoder.get(&s))
    //         .unwrap()
    //         .as_bstr())
    // );

    Ok(())
}

// encode mismatch #18: [29871, 1, 29961, 29925, 3035, 29962, 2]
// expected        #18: [1, 518, 29925, 3035, 29962, 2]
// decode mismatch #18: "<s>[PAD]</s>"
// expected        #18: "<s> [PAD]</s>"

// encode mismatch #18: [29871, 29966, 29879, 29958, 29961, 29925, 3035, 29962, 829, 29879, 29958]
// expected        #18: [529, 29879, 24566, 29925, 3035, 29962, 829, 29879, 29958]
