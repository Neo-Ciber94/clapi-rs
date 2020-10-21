use crate::utils::{Then, Also};

/// Provides suggestions matches a value.
///
/// # Implementing SuggestionProvider:
/// Suggestion provided require the function `suggestions_for` where you
/// use the `source` values and returns the matches, or `None` if not match is found.
pub trait SuggestionProvider{
    /// Returns the suggestion values for the given `source` and `value`,
    /// or `None` if not suggestion if found.
    fn suggestions_for(&self, value: &str, source: &[String]) -> Option<Vec<String>>;

    /// Returns a suggestion message with the suggestion values.
    /// By default this method returns a message for when a single match is found and other
    /// when multiple matches are found.
    fn suggestion_message_for(&self, value: &str, source: &[String]) -> Option<String>{
        if let Some(suggestions) = self.suggestions_for(value, source){
            match suggestions.len(){
                0 => None,
                1 => Some(format!("\n\n\tDid you mean `{}`?\n", suggestions[0])),
                _ => {
                    let formatted_values = suggestions.join("\n");
                    Some(format!("Possible values: \n{}", formatted_values))
                }
            }
        }
        else{
            None
        }
    }
}

/// A default implementation of `SuggestionProvider` that returns a limited count of suggestions
/// using the `Levenshtein distance` algorithm.
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
///     .contains(&String::from("strawberry")));
///
/// assert!(provider.suggestions_for("maple", &source)
///     .unwrap()
///     .contains(&String::from("apple")));
/// ```
///
/// # See
/// https://en.wikipedia.org/wiki/Levenshtein_distance
pub struct DefaultSuggestionProvider { max_count: usize }
impl DefaultSuggestionProvider{
    /// Constructs a new `DefaultSuggestionProvider` that returns a max of 5 suggestions.
    pub const fn new() -> Self{
        DefaultSuggestionProvider { max_count: 5 }
    }

    /// Constructs a new `DefaultSuggestionProvider` that returns the specified
    /// max number of suggestions.
    pub fn with_max_count(max_count: usize) -> Self {
        assert!(max_count > 0);
        DefaultSuggestionProvider { max_count }
    }

    /// Returns the max number of suggestions.
    pub fn max_count(&self) -> usize{
        self.max_count
    }
}

impl SuggestionProvider for DefaultSuggestionProvider {
    fn suggestions_for(&self, value: &str, source: &[String]) -> Option<Vec<String>> {
        source.iter()
            .map(|s| (s, compute_levenshtein_distance_ignore_case(&value, &s)))
            .take(self.max_count)
            .collect::<Vec<_>>()
            .also_mut(|s| s.sort_by_key(|s| s.1))
            .iter()
            .map(|s| s.0)
            .cloned()
            .collect::<Vec<String>>()
            .then_apply(|v| Some(v))
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
///     .contains(&String::from("apple")));
/// ```
///
/// # See
/// https://en.wikipedia.org/wiki/Levenshtein_distance
pub struct SingleSuggestionProvider;
impl SuggestionProvider for SingleSuggestionProvider{
    fn suggestions_for(&self, value: &str, source: &[String]) -> Option<Vec<String>> {
        let mut current = None;
        let mut current_cost = usize::max_value();

        for s in source {
            let cost = compute_levenshtein_distance_ignore_case(&value, s);
            if cost < current_cost{
                current = Some(s);
                current_cost = cost;
            }
        }

        if let Some(result) = current.cloned(){
            Some(std::slice::from_ref(&result).to_vec())
        }
        else{
            None
        }
    }
}

/// Compute the `Levenshtein distance` between 2 `str`
///
/// # See
/// https://en.wikipedia.org/wiki/Levenshtein_distance
#[inline]
pub fn compute_levenshtein_distance_ignore_case(x: &str, y: &str) -> usize{
    compute_levenshtein_distance(x, y, true)
}

/// Compute the `Levenshtein distance` between 2 `str` with ignore case.
///
/// # See
/// https://en.wikipedia.org/wiki/Levenshtein_distance
pub fn compute_levenshtein_distance(x: &str, y: &str, ignore_case: bool) -> usize{
    if x == y {
        return 0;
    }

    let len_x = x.chars().count();
    let len_y = y.chars().count();

    if len_x == 0 { return len_y; }
    if len_y == 0 { return len_x; }

    #[inline(always)]
    fn calculate_cost(a: char, b: char) -> usize{
        if a == b { 0 } else { 1 }
    }

    #[inline(always)]
    fn min<T: Ord>(a: T, b: T, c: T) -> T{
        std::cmp::min( std::cmp::min(a, b), c)
    }

    let mut distance = vec![vec![0; len_y + 1]; len_x + 1];

    for i in 0..=len_x { distance[i][0] = i; }
    for j in 0..=len_y { distance[0][j] = j; }

    for i in 1..=len_x {
        for j in 1..=len_y {
            let mut c1 = x.chars().nth(i - 1).unwrap();
            let mut c2 = y.chars().nth(j - 1).unwrap();

            if ignore_case {
                c1 = c1.to_ascii_lowercase();
                c2 = c2.to_ascii_lowercase();
            }

            distance[i][j] = min(
                distance[i - 1][j] + 1,
                distance[i][j - 1] + 1,
                distance[i - 1][j - 1] + calculate_cost(c1, c2),
            );
        }
    }

    distance[len_x][len_y]
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn compute_levenshtein_distance_test(){
        assert_eq!(compute_levenshtein_distance_ignore_case("pop", "pop"), 0);
        assert_eq!(compute_levenshtein_distance_ignore_case("casa", "calle"), 3);
        assert_eq!(compute_levenshtein_distance_ignore_case("shot", "spot"), 1);
        assert_eq!(compute_levenshtein_distance_ignore_case("dad", "mom"), 3);
        assert_eq!(compute_levenshtein_distance_ignore_case("blueberry", "berry"), 4);
    }
}
