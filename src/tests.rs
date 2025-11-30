use alloc::borrow::Cow;
use core::fmt::Error;

use crate::{
    I18nString, NoResolver, Resolver, Translatable,
    builder::{I18n, I18nBuilder, WantsTemplate},
};

struct SimpleResolver;

impl Resolver for SimpleResolver {
    fn resolve<'s>(&'s self, fmt: Cow<'s, str>) -> Cow<'s, str> {
        match fmt.as_ref() {
            "resource changed" => "资源变更".into(),
            _ => fmt.into(),
        }
    }
}

#[test]
fn test_simple_i18n() {
    let resolver = SimpleResolver;

    let cases = [
        (
            "t!('pull {1} error: {0}', t!('resource changed'), '/file')",
            "pull /file error: 资源变更",
        ),
        ("t!('User {0} says: \\'Hello\\'!', 'Alice')", "User Alice says: 'Hello'!"),
        (
            "t!('Start -> {0} -> End', t!('Middle {0}', t!('Inner')))",
            "Start -> Middle Inner -> End",
        ),
        (
            "Error log: t!('File not found: {0}', 'config.json'). Please check.",
            "Error log: File not found: config.json. Please check.",
        ),
        ("t!(  'Trim {0}'  ,  'test'  )", "Trim test"),
        // 边界测试：没有参数的 t!
        ("t!('Just string')", "Just string"),
        // 边界测试：嵌套与普通文本混合
        ("Prefix t!('A') middle t!('B') suffix", "Prefix A middle B suffix"),
    ];

    for (input, expected) in cases {
        let output = super::transform(input, &resolver).unwrap();

        assert_eq!(output, expected);
    }
}

#[test]
#[cfg(feature = "std")]
fn bench_simple_i18n() {
    const N: usize = 1_000_000;

    let begin_at = std::time::Instant::now();

    for _ in 0..N {
        std::hint::black_box(test_simple_i18n());
    }

    let elapsed = begin_at.elapsed();
    println!("bench_simple_i18n: {:?}", elapsed / N as u32);
}

#[test]
fn test_builder() {
    enum I18nInnerError {
        IoError(&'static str),
        HasEscaped(&'static str),
    }

    impl I18n for I18nInnerError {
        fn build_i18n(&self, builder: I18nBuilder<WantsTemplate>) -> Result<(), Error> {
            match self {
                I18nInnerError::IoError(msg) => builder.template("io error: {0}")?.arg_display(msg)?.finish(),
                I18nInnerError::HasEscaped(msg) => builder.template("has escaped: \'{0}\'")?.arg_display(msg)?.finish(),
            }
        }
    }

    enum I18nError {
        InnerError(I18nInnerError),
        InvalidFormat,
    }

    impl I18n for I18nError {
        fn build_i18n(&self, builder: I18nBuilder<WantsTemplate>) -> Result<(), Error> {
            match self {
                I18nError::InnerError(err) => builder.template("inner error: {0}")?.arg_i18n(err)?.finish(),
                I18nError::InvalidFormat => builder.template("invalid format")?.finish(),
            }
        }
    }

    let cases = [
        (
            I18nError::InnerError(I18nInnerError::IoError("test")),
            "t!('inner error: {0}',t!('io error: {0}','test'))",
            "inner error: io error: test",
        ),
        (
            I18nError::InnerError(I18nInnerError::HasEscaped("\\test")),
            "t!('inner error: {0}',t!('has escaped: \\'{0}\\'','\\\\test'))",
            "inner error: has escaped: '\\test'",
        ),
        (I18nError::InvalidFormat, "t!('invalid format')", "invalid format"),
    ];

    for (input, expected_template, expected) in cases {
        let mut s = I18nString::alloc(64);
        input.build_i18n(I18nBuilder::new(&mut s)).unwrap();

        assert_eq!(&*s, expected_template);

        s.translate_in_place(&NoResolver).unwrap();

        assert_eq!(&*s, expected);
    }
}

#[test]
#[cfg(feature = "std")]
fn bench_builder() {
    const N: usize = 100_000;

    let begin_at = std::time::Instant::now();

    for _ in 0..N {
        std::hint::black_box(test_builder());
    }

    let elapsed = begin_at.elapsed();
    println!("bench_builder: {:?}", elapsed / N as u32);
}
