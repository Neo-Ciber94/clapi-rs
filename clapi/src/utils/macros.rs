/// Asserts that the given value is not empty by calling `is_empty` and returns the value.
#[macro_export]
macro_rules! assert_not_empty {
    ($value:expr) => {{
        let value = $value;
        assert!(!value.is_empty(), "`{}` is empty", stringify!(value));
        value
    }};

    ($value:expr, $args:tt) => {{
        let value = $value;
        assert!(!value.is_empty(), $args);
        value
    }};
}

/// Asserts the given `String` or `str` is not empty or consists only of whitespaces.
#[macro_export]
macro_rules! assert_not_blank {
    ($value:expr) => {{
        let value = $value;
        assert!(!value.trim().is_empty(), "`{}` is blank", stringify!(value));
        value
    }};

    ($value:expr, $args:tt) => {{
        let value = $value;
        assert!(!value.trim().is_empty(), $args);
        value
    }};
}

/// Asserts the given `String` or `str` is no empty or contains whitespaces.
#[macro_export]
macro_rules! assert_contains_no_whitespaces {
    ($value:expr) => {{
        let value = &$value;
        assert!(!value.trim().is_empty(), "`{}` is empty", stringify!($value));
        assert!(value.trim().chars().all(|c| !c.is_whitespace()), "`{}` must not contains whitespaces", stringify!($value));
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn assert_not_empty_test1() {
        assert_not_empty!(String::from("Hello World"));
        assert_not_empty!("Hello World");
    }

    #[test]
    #[should_panic]
    fn assert_not_empty_test2() {
        let _value = assert_not_empty!(String::new());
    }

    #[test]
    #[should_panic]
    fn assert_not_empty_test3() {
        let _value = assert_not_empty!(String::from(""));
    }

    #[test]
    fn assert_not_blank_test1() {
        assert_not_blank!("Hello");
        assert_not_blank!("Hello World");
        assert_not_blank!("   a");
    }

    #[test]
    #[should_panic]
    fn assert_not_blank_test2() {
        assert_not_blank!("");
    }

    #[test]
    #[should_panic]
    fn assert_not_blank_test3() {
        assert_not_blank!("  ");
    }

    #[test]
    #[should_panic]
    fn assert_not_blank_test4() {
        assert_not_blank!("\t");
    }

    #[test]
    #[should_panic]
    fn assert_not_blank_test5() {
        assert_not_blank!("\n");
    }

    #[test]
    #[should_panic(expected="`name` must not contains whitespaces")]
    fn assert_contains_not_whitespaces_test1(){
        let name = "Hello World";
        let _ = assert_contains_no_whitespaces!(name);
    }

    #[test]
    #[should_panic(expected="`\" \"` is empty")]
    fn assert_contains_not_whitespaces_test2(){
        let _ = assert_contains_no_whitespaces!(" ");
    }
}
