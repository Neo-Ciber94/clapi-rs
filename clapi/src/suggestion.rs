use std::num::NonZeroUsize;

/// Represents a suggestion for an invalid `command` or `option`
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Suggestion {
    /// The suggested value.
    pub value: String,
    /// The similarity between the suggested value and the invalid one.
    pub similarity: f32,
}

/// Configuration for the suggestions.
#[derive(Debug, Clone)]
pub struct SuggestionSource {
    /// Max number of suggestion message.
    pub max_count: NonZeroUsize,
    /// Ignore case when computing suggestion messages.
    pub ignore_case: bool,
    /// Min similarity to consider for a suggestion message.
    pub min_similarity: f32,
    /// Provides the message for the suggestions.
    pub message: fn(Vec<Suggestion>) -> Option<String>,
}

impl Default for SuggestionSource {
    #[inline]
    fn default() -> Self {
        SuggestionSource::new()
    }
}

impl SuggestionSource {
    /// Constructs a new `SuggestionProvider`
    #[inline]
    pub fn new() -> Self {
        SuggestionSource {
            max_count: NonZeroUsize::new(1).unwrap(),
            ignore_case: true,
            min_similarity: 0.0,
            message: default_suggestion_message
        }
    }

    /// Returns a suggestion message for the `value` from the `source` values
    pub fn suggestions_for(&self, value: &str, source: &[String]) -> Vec<Suggestion> {
        suggestions_for(
            self.max_count,
            self.ignore_case,
            self.min_similarity,
            value,
            source
        )
    }

    /// Returns a suggestion message for the given suggestions.
    pub fn message_for(&self, values: Vec<Suggestion>) -> Option<String> {
        (self.message)(values)
    }
}

// Default suggestion message handler
fn default_suggestion_message(suggestions: Vec<Suggestion>) -> Option<String> {
    const INDENT: &str = "      ";

    match suggestions.len() {
        0 => None,
        // Did you mean `value`?
        1 => Some(format!("{}Did you mean `{}`?", INDENT, suggestions[0].value)),
        _ => {
            let mut values : String = suggestions[..suggestions.len() - 1]
                .into_iter()
                .map(|s| format!("`{}`", s.value))
                .collect::<Vec<String>>()
                .join(", ");

            values.push_str(format!(" or `{}`", suggestions.last().unwrap().value).as_str());

            // Did you mean `1`, `2` or `3`?
            Some(format!("{}Did you mean any of {}?", INDENT, values))
        }
    }
}

/// Returns a `Vec` of similar values to `value` using the given `source` values.
///
/// # Parameters
/// * `max_count` - Max number of suggestions to return.
/// * `ignore_case` - If ignore case when comparing the values.
/// * `min_similarity` - The min similarity expected between the values, from 0 to 1.
/// * `value` - The value to compare.
/// * `source` - The values to compare with.
pub fn suggestions_for(
    max_count: NonZeroUsize,
    ignore_case: bool,
    min_similarity: f32,
    value: &str,
    source: &[String],
) -> Vec<Suggestion> {
    debug_assert!(min_similarity >= 0_f32 && min_similarity <= 1_f32);
    let mut result = Vec::new();

    for s in source {
        let cost = compute_levenshtein_distance(value, s, ignore_case);
        let similarity = 1_f32 - (cost as f32 / std::cmp::max(value.len(), s.len()) as f32);

        if similarity >= min_similarity {
            result.push(Suggestion {
                value: s.clone(),
                similarity,
            });
        }

        if result.len() == max_count.get() {
            break;
        }
    }

    result.sort_by(|x, y| x.similarity.partial_cmp(&y.similarity).unwrap());
    result
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
