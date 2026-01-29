use alloc::{borrow::Cow, format, string::ToString};
use core::str::FromStr;

use crate::{I18nString, Resolver};

#[test]
fn test_parse() {
    let cases = [
        (
            "t!('pull {1} error: {0}', t!('resource changed'), '/file')",
            I18nString::template(
                "pull {1} error: {0}",
                [I18nString::template("resource changed", []), I18nString::literal("/file")],
            ),
        ),
        (
            "t!('User {0} says: \\'Hello\\'!', 'Alice')",
            I18nString::template("User {0} says: \'Hello\'!", [I18nString::literal("Alice")]),
        ),
        (
            "t!('Start -> {0} -> End', t!('Middle {0}', t!('Inner')))",
            I18nString::template(
                "Start -> {0} -> End",
                [I18nString::template("Middle {0}", [I18nString::template("Inner", [])])],
            ),
        ),
        (
            "t!(  'Trim {0}'  ,  'test'  )",
            I18nString::template("Trim {0}", [I18nString::literal("test")]),
        ),
        ("t!('No Args')", I18nString::template("No Args", [])),
        ("'just string'", I18nString::literal("just string")),
        (
            "t!('{0} {1} {2} {3}', 'a', 'b', 'c', 'd')",
            I18nString::template(
                "{0} {1} {2} {3}",
                [
                    I18nString::literal("a"),
                    I18nString::literal("b"),
                    I18nString::literal("c"),
                    I18nString::literal("d"),
                ],
            ),
        ),
        (
            "t!('Path: {0}', 'C:\\\\Program Files')",
            I18nString::template("Path: {0}", [I18nString::literal("C:\\Program Files")]),
        ),
        (
            "t!('Empty: {0}', '')",
            I18nString::template("Empty: {0}", [I18nString::literal("")]),
        ),
        ("t!('')", I18nString::template("", [])),
        (
            "t!('Special chars: {0}', '!@#$%^&*()_+-=[]{}|;:,.<>?')",
            I18nString::template("Special chars: {0}", [I18nString::literal("!@#$%^&*()_+-=[]{}|;:,.<>?")]),
        ),
        (
            "t!('Level 1: {0}', t!('Level 2: {0}', t!('Level 3: {0}', t!('Level 4'))))",
            I18nString::template(
                "Level 1: {0}",
                [I18nString::template(
                    "Level 2: {0}",
                    [I18nString::template("Level 3: {0}", [I18nString::template("Level 4", [])])],
                )],
            ),
        ),
        (
            "t!('{0} and {1}', t!('Nested A'), t!('Nested B'))",
            I18nString::template(
                "{0} and {1}",
                [I18nString::template("Nested A", []), I18nString::template("Nested B", [])],
            ),
        ),
        (
            "t!('Item {0} of {1}: {2}', '1', '10', t!('Description: {0}', 'Test'))",
            I18nString::template(
                "Item {0} of {1}: {2}",
                [
                    I18nString::literal("1"),
                    I18nString::literal("10"),
                    I18nString::template("Description: {0}", [I18nString::literal("Test")]),
                ],
            ),
        ),
        ("t!('Line 1\\nLine 2')", I18nString::template("Line 1\nLine 2", [])),
        (
            "t!('Tab\\tSeparated\\tValues')",
            I18nString::template("Tab\tSeparated\tValues", []),
        ),
        ("t!('Mix: \\'\\n\\t\\\\')", I18nString::template("Mix: '\n\t\\", [])),
        (
            "t!('{2}, {1}, {0}', 'third', 'second', 'first')",
            I18nString::template(
                "{2}, {1}, {0}",
                [
                    I18nString::literal("third"),
                    I18nString::literal("second"),
                    I18nString::literal("first"),
                ],
            ),
        ),
        (
            "t!('Hello {0}, welcome {0}!', 'Guest')",
            I18nString::template("Hello {0}, welcome {0}!", [I18nString::literal("Guest")]),
        ),
    ];

    for (input, expected) in cases {
        let output = I18nString::from_str(input).unwrap();
        assert_eq!(output, expected);
    }
}

#[test]
fn test_format_and_parse() {
    let cases = [
        I18nString::literal("just string"),
        I18nString::template("Hello {0}", [I18nString::literal("World")]),
        I18nString::template("Hello {0} {1}", [I18nString::literal("World"), I18nString::literal("!")]),
        I18nString::template("Nest {0}", [I18nString::template("Inner {0}", [I18nString::literal("Test")])]),
        I18nString::template("Empty: {0}", [I18nString::literal("")]),
        I18nString::template("Special chars: {0}", [I18nString::literal("!@#$%^&*()_+-=[]{}|;:,.<>?")]),
        I18nString::template("Newline: {0}\n", [I18nString::literal("\n")]),
    ];

    for case in cases {
        let formatted = case.to_string();
        let parsed = I18nString::from_str(&formatted).expect(&format!("Failed to parse formatted string: {formatted}"));
        assert_eq!(parsed, case);
    }
}

#[test]
fn test_translate() {
    struct SimpleResolver;

    impl Resolver for SimpleResolver {
        fn resolve<'s>(&'s self, fmt: &'s str) -> Cow<'s, str> {
            match fmt.as_ref() {
                "resource changed" => "资源变更".into(),
                "io error: {0}" => "IO 错误: {0}".into(),
                _ => fmt.into(),
            }
        }
    }

    let cases = [
        ("t!('resource changed')", "资源变更"),
        ("t!('io error: {0}', 'file not found')", "IO 错误: file not found"),
        ("t!('Empty: {0}', '')", "Empty: "),
        (
            "t!('Special chars: {0}', '!@#$%^&*()_+-=[]{}|;:,.<>?')",
            "Special chars: !@#$%^&*()_+-=[]{}|;:,.<>?",
        ),
        ("t!('Newline: {0}\n', '\n')", "Newline: \n\n"),
    ];

    let resolver = SimpleResolver;
    for (input, expected) in cases {
        let output = I18nString::from_str(input).unwrap().translate(&resolver);
        assert_eq!(output, expected);
    }
}
