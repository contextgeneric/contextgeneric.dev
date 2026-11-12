//! Runnable check for the code shown on the front page, `src/pages/index.tsx`.
//!
//! The front page belongs to no `docs/` section, so it gets its own test binary, as the Quickstart
//! does. The page's one compiling block is the hero's encoder pair under CGP. The rejected half of the
//! hero, the same pair without CGP, cannot compile, so it is a compile-fail fixture at
//! `tests/compile_fail/front_page/`.
//!
//! The text between each pair of `front-page:` marker comments is exactly what the page shows, so a
//! test can compare it with `index.tsx`. Everything outside the markers is what the page omits: the
//! imports and the checks. The rejected
//! block starts at its fixture's first line, so the page's quoted `E0119` names the lines the page
//! numbers; that fixture carries only the end marker.

// The hero's CGP block. The page omits these two imports.
mod first_screen {
    use cgp::prelude::*;
    use core::fmt::Display;

    // front-page:cgp-block:begin
    #[cgp_component(Encoder)]
    pub trait CanEncode {
        fn encode(&self) -> Vec<u8>;
    }

    #[cgp_impl(new EncodeAsText)]
    #[uses(Display)]
    impl Encoder {
        fn encode(&self) -> Vec<u8> { self.to_string().into_bytes() }
    }

    #[cgp_impl(new EncodeAsBytes)]
    #[uses(AsRef<[u8]>)]
    impl Encoder {
        fn encode(&self) -> Vec<u8> { self.as_ref().to_vec() }
    }

    // Each type names the implementation it uses:
    delegate_components! { u64     { EncoderComponent: EncodeAsText } }
    delegate_components! { String  { EncoderComponent: EncodeAsText } }
    delegate_components! { Vec<u8> { EncoderComponent: EncodeAsBytes } }
    // front-page:cgp-block:end

    // The page does not show checks; these prove each wiring resolves rather than merely parses.
    check_components! { u64 { EncoderComponent } }
    check_components! { String { EncoderComponent } }
    check_components! { Vec<u8> { EncoderComponent } }

    // Property: for each wired type, `encode` runs the provider the wiring names. `u64` and `String`
    // take `EncodeAsText`, which encodes the `Display` text; `Vec<u8>` takes `EncodeAsBytes`, which
    // copies the bytes. `String` is the witness: it satisfies both bounds, so its result shows which
    // provider the wiring chose.
    #[test]
    fn each_type_encodes_with_the_provider_it_names() {
        assert_eq!(42u64.encode(), b"42");
        assert_eq!("hi".to_owned().encode(), b"hi");
        assert_eq!(vec![0xffu8, 0x00].encode(), vec![0xff, 0x00]);
    }
}

// Property: the text the page shows between each pair of `front-page:` markers equals the text this
// crate compiles between the same markers. The page keeps its blocks as template literals in
// `src/pages/index.tsx`, which no compiler reads, so without this check the page could drift from
// the code that proves it.
mod page_text {
    use std::fs;
    use std::path::Path;

    /// The lines strictly between `// front-page:<name>:begin` and `// front-page:<name>:end`.
    fn region<'a>(source: &'a str, name: &str) -> Vec<&'a str> {
        let begin = format!("// front-page:{name}:begin");
        let end = format!("// front-page:{name}:end");
        let lines: Vec<&str> = source.lines().collect();
        let start = lines.iter().position(|l| l.trim() == begin).expect("begin marker") + 1;
        let stop = lines.iter().position(|l| l.trim() == end).expect("end marker");
        lines[start..stop].to_vec()
    }

    /// The lines from the start of a file up to `// front-page:<name>:end`.
    fn leading_region<'a>(source: &'a str, name: &str) -> Vec<&'a str> {
        let end = format!("// front-page:{name}:end");
        let lines: Vec<&str> = source.lines().collect();
        let stop = lines.iter().position(|l| l.trim() == end).expect("end marker");
        lines[..stop].to_vec()
    }

    /// A Rust region, with the indentation of the module it sits in removed.
    fn rust_text(lines: &[&str]) -> String {
        let indent = lines
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start().len())
            .min()
            .unwrap_or(0);
        lines
            .iter()
            .map(|l| if l.trim().is_empty() { "" } else { &l[indent..] })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The template literal a TSX region declares, with its escapes undone.
    fn tsx_text(lines: &[&str]) -> String {
        let joined = lines.join("\n");
        let open = joined.find('`').expect("opening backtick") + 1;
        let close = joined.rfind('`').expect("closing backtick");
        joined[open..close].replace("\\`", "`").replace("\\$", "$").replace("\\\\", "\\")
    }

    fn read(relative: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    #[test]
    fn the_page_shows_the_code_this_crate_compiles() {
        let page = read("../src/pages/index.tsx");
        let compiled = read("tests/front_page.rs");
        let rejected = read("tests/compile_fail/front_page/rust_accepts_only_one.rs");
        assert_eq!(
            tsx_text(&region(&page, "rejected-block")),
            rust_text(&leading_region(&rejected, "rejected-block")),
            "the page's `rejected-block` differs from the compiled code"
        );
        assert_eq!(
            tsx_text(&region(&page, "cgp-block")),
            rust_text(&region(&compiled, "cgp-block")),
            "the page's `cgp-block` differs from the compiled code"
        );
    }

    // Property: the diagnostic the page quotes under the rejected block equals the compiler's output
    // for that block, as the compile-fail snapshot records it, without the location line, which names
    // the fixture's path rather than anything on the page.
    #[test]
    fn the_page_quotes_the_compiler_error() {
        let page = read("../src/pages/index.tsx");
        let snapshot = read("tests/compile_fail/front_page/rust_accepts_only_one.stderr");
        let expected = snapshot
            .lines()
            .filter(|l| !l.trim_start().starts_with("--> "))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            tsx_text(&region(&page, "rejected-diagnostic")),
            expected.trim_end(),
            "the page's quoted diagnostic differs from the compile-fail snapshot"
        );
    }
}
