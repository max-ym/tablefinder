use std::borrow::Cow;

/// A kind of column.
pub trait ColumnKind {
    /// Assess how much given header is similar to what we can expect for this kind of column.
    fn assess_header(&self, header: &str) -> f32;

    /// Assess how much given value is similar to what we can expect for this kind of column.
    fn assess_value(&self, value: &str) -> f32;
}

#[derive(Debug, Clone, Copy)]
pub struct Assessment {
    pub similarity: f32,
    pub position: usize,
}

impl Assessment {
    /// Assess headers for match scores for each column kind.
    ///
    /// # Arguments
    /// `column_set` - A set of column kinds to assess.
    /// `headers` - An iterator of headers to assess.
    ///
    /// # Returns
    /// A vector of vectors of assessments for each column kind.
    /// Outer vector is for each header, inner vector is for each column kind.
    pub fn for_headers<'headers>(
        column_set: &[impl ColumnKind],
        headers: impl Iterator<Item = &'headers (impl AsRef<str> + 'headers)>,
    ) -> Vec<Vec<Self>> {
        headers
            .enumerate()
            .map(|(position, header)| {
                let mut vec = Vec::with_capacity(column_set.len());
                for kind in column_set {
                    vec.push(Assessment {
                        similarity: kind.assess_header(header.as_ref()),
                        position,
                    });
                }
                vec
            })
            .collect()
    }

    /// Assess rows which may be headers for match scores for each column kind.
    pub fn header_rows_iter<'headers, Rows, Row, Ck, Str>(
        column_set: &[Ck],
        rows: Rows,
    ) -> impl Iterator<Item = Vec<Vec<Self>>> + use<'headers, '_, Rows, Row, Ck, Str>
    where
        Str: AsRef<str> + 'headers,
        Row: Iterator<Item = &'headers Str>,
        Rows: Iterator<Item = Row>,
        Ck: ColumnKind,
    {
        rows.map(move |row| Assessment::for_headers(column_set, row))
    }
}

/// Configuration for predefined assessment algorithms.
#[derive(Debug, Clone, Copy)]
pub struct SimpleAssessor {
    /// Whether to consider case sensitivity. If true, case is treated as distinct characters.
    /// Otherwise, all characters are converted to lowercase.
    /// Dictionary is expected to contain lowercase characters.
    pub is_case_sensitive: bool,

    /// Whether to consider digit sensitivity. If true, digits are treated as distinct characters.
    /// Otherwise, all digits are converted to 0s and dictionary is expected to
    /// contain 0s instead of digits.
    pub is_digit_sensitive: bool,

    /// Whether to suppress numeric values. If true, all continuous numbers
    /// are replaced with a single 0.
    /// On true, overrides [is_digit_sensitive](Self::is_digit_sensitive).
    /// Dictionary is expected to contain 0s instead of digits, and
    /// all 0 should be surrounded by non-digits (or start/end of string).
    pub is_number_reduced: bool,

    /// Whether to suppress alphabetic values. If true, all continuous alphabets
    /// are replaced with a single 'a'.
    /// On true, overrides [is_case_sensitive](Self::is_case_sensitive).
    /// Dictionary is expected to contain 'a's instead of alphabets, and
    /// all 'a's should be surrounded by non-alphabets (or start/end of string).
    pub is_alpha_reduced: bool,
}

impl Default for SimpleAssessor {
    fn default() -> Self {
        Self {
            is_case_sensitive: false,
            is_digit_sensitive: false,
            is_number_reduced: true,
            is_alpha_reduced: false,
        }
    }
}

impl SimpleAssessor {
    pub fn with_dict<S: AsRef<str>>(
        self,
        value: impl AsRef<str>,
        dict: impl Iterator<Item = S>,
    ) -> f64 {
        let value = if self.is_alpha_reduced {
            // Replace all continuous alphabets with 'a', expecting dictionary to present alphabets as 'a's
            Cow::Owned(reduce(value.as_ref(), char::is_alphabetic, 'a'))
        } else if self.is_case_sensitive {
            Cow::Borrowed(value.as_ref())
        } else {
            Cow::Owned(value.as_ref().to_lowercase())
        };
        let value = if self.is_number_reduced {
            // Replace all continuous digits with 0, expecting dictionary to present digits as 0s
            Cow::Owned(reduce(value.as_ref(), |c| char::is_digit(c, 10), '0'))
        } else if self.is_digit_sensitive {
            // Replace all digits with 0, expecting dictionary to present digits as 0s
            Cow::Owned(
                value
                    .as_ref()
                    .chars()
                    .map(|c| if c.is_digit(10) { '0' } else { c })
                    .collect(),
            )
        } else {
            value
        };

        let mut max = 0.0;
        for variant in dict {
            if cfg!(debug_assertions) {
                if self.is_alpha_reduced {
                    assert_reduction(variant.as_ref(), char::is_alphabetic);
                } else if self.is_case_sensitive {
                    assert!(variant.as_ref().chars().all(|c| c.is_lowercase()));
                }
                if self.is_number_reduced {
                    assert_reduction(variant.as_ref(), |c| char::is_digit(c, 10));
                } else if self.is_digit_sensitive {
                    assert!(variant
                        .as_ref()
                        .chars()
                        .all(|c| c == '0' || !c.is_digit(10)));
                }
            }

            let similarity = strsim::jaro(&value, variant.as_ref());
            if max < similarity {
                max = similarity;
            }
        }
        max
    }
}

fn reduce(value: &str, criteria: impl Fn(char) -> bool, with: char) -> String {
    let mut chars = value.chars();
    let mut result = String::with_capacity(value.len());
    let mut last_was = false;
    while let Some(c) = chars.next() {
        if criteria(c) {
            if !last_was {
                result.push(with);
                last_was = true;
            }
        } else {
            result.push(c);
            last_was = false;
        }
    }
    result.shrink_to_fit();
    result
}

#[cfg(debug_assertions)]
#[track_caller]
fn assert_reduction(value: &str, criteria: impl Fn(char) -> bool) {
    let mut last_was = false;
    for c in value.chars() {
        if criteria(c) {
            assert!(!last_was, "reduction not performed for `{value}`");
            last_was = true;
        } else {
            last_was = false;
        }
    }
}

#[cfg(test)]
mod tests;
