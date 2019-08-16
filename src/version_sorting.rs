//! Simple version-sorting trait and function.

pub trait Versioned {
    fn version(&self) -> &str;
}

// Sort from oldest to latest.
pub fn sort_versions<V: Versioned>(input: &mut [V]) {
    input.sort_by(|a, b| human_sort::compare(a.version(), b.version()));
}

/// Sort from latest to oldest.
#[cfg(test)] // Only used in tests
pub fn sort_versions_reverse<V: Versioned>(input: &mut [V]) {
    input.sort_by(|a, b| human_sort::compare(b.version(), a.version()));
}

#[cfg(feature = "fwupd")]
impl Versioned for fwupd_dbus::Release {
    fn version(&self) -> &str { &self.version }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[derive(Debug, PartialEq)]
    struct Foo {
        version: String,
    }

    impl Versioned for Foo {
        fn version(&self) -> &str { &self.version }
    }

    fn test_input() -> Vec<Foo> {
        vec![
            Foo { version: "0.2.10.0".into() },
            Foo { version: "0.2.11.0".into() },
            Foo { version: "0.2.8.1".into() },
            Foo { version: "0.2.9.0".into() },
            Foo { version: "0.l2.12.0".into() },
        ]
    }

    #[test]
    fn sort_versions_test() {
        let mut input = test_input();

        let expected = vec![
            Foo { version: "0.2.8.1".into() },
            Foo { version: "0.2.9.0".into() },
            Foo { version: "0.2.10.0".into() },
            Foo { version: "0.2.11.0".into() },
            Foo { version: "0.l2.12.0".into() },
        ];

        sort_versions(&mut input);
        assert_eq!(input, expected);
    }

    #[test]
    fn sort_versions_reverse_test() {
        let mut input = test_input();

        let expected = vec![
            Foo { version: "0.l2.12.0".into() },
            Foo { version: "0.2.11.0".into() },
            Foo { version: "0.2.10.0".into() },
            Foo { version: "0.2.9.0".into() },
            Foo { version: "0.2.8.1".into() },
        ];

        sort_versions_reverse(&mut input);
        assert_eq!(input, expected);
    }
}
