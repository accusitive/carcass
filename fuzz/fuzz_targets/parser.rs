#![no_main]

use std::{
    env,
    fs,
    hash::{
        self,
        Hash,
        Hasher,
    },
    hint,
    path,
};

use cab::syntax::parse;
use libfuzzer_sys::{
    fuzz_target,
    Corpus,
};
use yansi::{
    Condition,
    Paint as _,
};

fuzz_target!(|data: &str| -> Corpus {
    let parse = hint::black_box(parse(data));

    if !env::var("FUZZ_PARSER_SAVE_VALID")
        .is_ok_and(|value| !matches!(value.as_ref(), "" | "0" | "false"))
    {
        return Corpus::Keep;
    }

    yansi::whenever(Condition::TTY_AND_COLOR);

    let Ok(node) = parse.result() else {
        return Corpus::Reject;
    };

    print!("found a valid parse!");

    let syntax = format!("{syntax:#?}", syntax = *node);

    let syntax_hash = {
        let mut hasher = hash::DefaultHasher::new();
        syntax.hash(&mut hasher);
        hasher.finish()
    };

    let base_file = format!("{syntax_hash:x}");

    let (data_file, syntax_file) = {
        let root = path::Path::new("cab-syntax/test/data");
        fs::create_dir_all(root).unwrap();

        (
            root.join(base_file.clone() + ".cab"),
            root.join(base_file.clone() + ".expect"),
        )
    };

    if data_file.exists() {
        println!(
            " seems like it was already known before, skipping writing {name}",
            name = base_file.yellow().bold()
        );

        Corpus::Reject
    } else {
        println!(" wrote it to {name}", name = base_file.green().bold());
        fs::write(data_file, data).unwrap();
        fs::write(syntax_file, syntax).unwrap();

        Corpus::Keep
    }
});
