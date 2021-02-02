use std::borrow::Borrow;

/// Provides suggestions matches a value.
///
/// # Implementing SuggestionProvider:
/// Suggestion provided require the function `suggestions_for` where you
/// use the `source` values and returns the matches, or `None` if not match is found.
pub trait SuggestionProvider {
    /// Returns the suggestion values for the given `source` and `value`,
    /// or `None` if not suggestion if found.
    fn suggestions_for(&self, value: &str, source: &[String]) -> Option<SuggestionResult>;

    /// Provides a message for the given suggestions.
    fn suggestion_message_for(&self, suggestions: SuggestionResult) -> Option<String> {
        match suggestions {
            // Did you mean `{}`?
            SuggestionResult::Value(s) => Some(format!("\tDid you mean `{}`?", s)),
            // Did you mean any of `{0}`, `{1}`, `{2}`?
            SuggestionResult::List(values) => {
                let values = values
                    .into_iter()
                    .map(|s| format!("`{}`", s))
                    .collect::<Vec<String>>()
                    .join(", ");

                Some(format!("\tDid you mean any of {}?", values))
            }
        }
    }
}

/// A default implementation of `SuggestionProvider` that returns a max count of suggestions.
///
/// # Example
/// ```rust
/// use std::string::ToString;
/// use clapi::suggestion::{DefaultSuggestionProvider, SuggestionProvider};
///
/// let source = vec!["apple", "banana", "mango", "strawberry"].iter()
///     .map(ToString::to_string)
///     .collect::<Vec<String>>();
///
/// let provider = DefaultSuggestionProvider::new();
///
/// assert!(provider.suggestions_for("berry", &source)
///     .unwrap()
///     .contains("strawberry"));
///
/// assert!(provider.suggestions_for("maple", &source)
///     .unwrap()
///     .contains("apple"));
/// ```
///
/// # See
/// https://en.wikipedia.org/wiki/Levenshtein_distance
pub struct DefaultSuggestionProvider {
    max_count: usize,
}
impl DefaultSuggestionProvider {
    /// Constructs a new `DefaultSuggestionProvider` that returns a max of 5 suggestions.
    pub const fn new() -> Self {
        DefaultSuggestionProvider { max_count: 5 }
    }

    /// Constructs a new `DefaultSuggestionProvider` that returns the specified
    /// max number of suggestions.
    pub fn with_max_count(max_count: usize) -> Self {
        assert!(max_count > 0);
        DefaultSuggestionProvider { max_count }
    }

    /// Returns the max number of suggestions.
    pub fn max_count(&self) -> usize {
        self.max_count
    }
}

impl SuggestionProvider for DefaultSuggestionProvider {
    fn suggestions_for(&self, value: &str, source: &[String]) -> Option<SuggestionResult> {
        let mut values = source
            .iter()
            .map(|s| (s, compute_levenshtein_distance_ignore_case(&value, &s)))
            .take(self.max_count)
            .collect::<Vec<_>>();

        // Sorts the values by weight
        values.sort_by_key(|s| s.1);

        let mut result = values
            .iter()
            .map(|s| s.0)
            .cloned()
            .collect::<Vec<String>>();

        if result.is_empty() {
            None
        } else if result.len() == 1 {
            Some(SuggestionResult::Value(result.swap_remove(0)))
        } else {
            Some(SuggestionResult::List(result))
        }
    }
}

/// A default implementation of `SuggestionProvider` that returns a single suggestion
/// using the `Levenshtein distance` algorithm.
///
/// # Example
/// ```rust
/// use std::string::ToString;
/// use clapi::suggestion::{SingleSuggestionProvider, SuggestionProvider};
///
/// let source = vec!["apple", "banana", "mango", "strawberry"].iter()
///     .map(ToString::to_string)
///     .collect::<Vec<String>>();
///
/// let provider = SingleSuggestionProvider;
///
/// assert!(provider.suggestions_for("maple", &source)
///     .unwrap()
///     .contains("apple"));
/// ```
///
/// # See
/// https://en.wikipedia.org/wiki/Levenshtein_distance
pub struct SingleSuggestionProvider;
impl SuggestionProvider for SingleSuggestionProvider {
    fn suggestions_for(&self, value: &str, source: &[String]) -> Option<SuggestionResult> {
        let mut current = None;
        let mut current_cost = usize::max_value();

        for s in source {
            let cost = compute_levenshtein_distance_ignore_case(&value, s);
            if cost < current_cost {
                current = Some(s);
                current_cost = cost;
            }
        }

        if let Some(result) = current.cloned() {
            Some(SuggestionResult::Value(result))
        } else {
            None
        }
    }
}

/// Represents the values of a suggestion.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SuggestionResult {
    /// A single suggestion value.
    Value(String),
    /// A list of suggested values.
    List(Vec<String>),
}

impl SuggestionResult {
    pub fn contains<S: Borrow<str>>(&self, value: S) -> bool {
        match self {
            SuggestionResult::Value(s) => s == value.borrow(),
            SuggestionResult::List(values) => values.iter().any(|s| s == value.borrow()),
        }
    }

    pub fn map<F: Fn(String) -> String>(self, f: F) -> SuggestionResult {
        match self {
            SuggestionResult::Value(s) => SuggestionResult::Value(f(s)),
            SuggestionResult::List(values) => {
                SuggestionResult::List(values.into_iter().map(f).collect())
            }
        }
    }
}

/// Compute the `Levenshtein distance` between 2 `str`
///
/// # See
/// https://en.wikipedia.org/wiki/Levenshtein_distance
#[inline]
#[doc(hidden)]
pub fn compute_levenshtein_distance_ignore_case(x: &str, y: &str) -> usize {
    compute_levenshtein_distance(x, y, true)
}

/// Compute the `Levenshtein distance` between 2 `str` with ignore case.
///
/// # Implementation from
/// https://github.com/wooorm/levenshtein-rs/blob/main/src/lib.rs
///
/// # See
/// https://en.wikipedia.org/wiki/Levenshtein_distance
#[doc(hidden)]
pub fn compute_levenshtein_distance(a: &str, b: &str, ignore_case: bool) -> usize {
    if a == b {
        return 0;
    }

    let length_a = a.chars().count();
    let length_b = b.chars().count();

    if length_a == 0 {
        return length_b;
    }

    if length_b == 0 {
        return length_a;
    }

    #[inline(always)]
    fn equals(a: char, b: char, ignore_case: bool) -> bool {
        if ignore_case {
            a.eq_ignore_ascii_case(&b)
        } else {
            a == b
        }
    }

    let mut cache: Vec<usize> = (1..=length_a).collect::<Vec<_>>();
    let mut result = 0;
    let mut distance_a: usize;
    let mut distance_b: usize;

    for (index_b, char_b) in b.chars().enumerate() {
        result = index_b;
        distance_a = index_b;

        for (index_a, char_a) in a.chars().enumerate() {
            distance_b = if equals(char_a, char_b, ignore_case) {
                distance_a
            } else {
                distance_a + 1
            };

            distance_a = cache[index_a];

            result = if distance_a > result {
                if distance_b > result {
                    result + 1
                } else {
                    distance_b
                }
            } else if distance_b > distance_a {
                distance_a + 1
            } else {
                distance_b
            };

            cache[index_a] = result;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_levenshtein_distance_test() {
        assert_eq!(compute_levenshtein_distance_ignore_case("pop", "pop"), 0);
        assert_eq!(compute_levenshtein_distance_ignore_case("casa", "calle"), 3);
        assert_eq!(compute_levenshtein_distance_ignore_case("shot", "spot"), 1);
        assert_eq!(compute_levenshtein_distance_ignore_case("dad", "mom"), 3);
        assert_eq!(
            compute_levenshtein_distance_ignore_case("blueberry", "berry"),
            4
        );
    }
}
