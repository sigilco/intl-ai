use crate::error::{Error, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs;
use std::path::Path;

/// Dotted-key glob selection for `--keys` / `--keys-file` and spec arguments
/// (cargo update -p precedent). Empty spec list matches everything.
/// Keys contain no `/`, so `auth.*` also matches `auth.deep.x` (subtree).
#[derive(Debug, Clone)]
pub struct KeySelector {
    specs: Option<Vec<String>>,
    set: Option<GlobSet>,
}

impl KeySelector {
    pub fn any() -> Self {
        Self {
            specs: None,
            set: None,
        }
    }

    pub fn from_specs(specs: &[String]) -> Result<Self> {
        let mut patterns = Vec::new();
        for spec in specs {
            for part in spec.split(',') {
                let p = part.trim();
                if !p.is_empty() {
                    patterns.push(p.to_string());
                }
            }
        }
        Self::from_patterns(&patterns)
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let patterns: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect();
        Self::from_patterns(&patterns)
    }

    /// Union of two selectors. `any` absorbs (an empty spec list means
    /// match-everything).
    pub fn merge(self, other: Self) -> Result<Self> {
        match (self.specs, other.specs) {
            (None, _) | (_, None) => Ok(Self::any()),
            (Some(mut a), Some(b)) => {
                a.extend(b);
                Self::from_patterns(&a)
            }
        }
    }

    fn from_patterns(patterns: &[String]) -> Result<Self> {
        if patterns.is_empty() {
            return Ok(Self::any());
        }
        let mut builder = GlobSetBuilder::new();
        for p in patterns {
            let glob =
                Glob::new(p).map_err(|e| Error::Message(format!("invalid key glob '{p}': {e}")))?;
            builder.add(glob);
        }
        let set = builder
            .build()
            .map_err(|e| Error::Message(format!("invalid key globs: {e}")))?;
        Ok(Self {
            specs: Some(patterns.to_vec()),
            set: Some(set),
        })
    }

    pub fn matches(&self, key: &str) -> bool {
        match &self.set {
            None => true,
            Some(s) => s.is_match(key),
        }
    }

    pub fn is_any(&self) -> bool {
        self.set.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtree_glob() {
        let sel = KeySelector::from_specs(&["auth.*".to_string()]).unwrap();
        assert!(sel.matches("auth.login"));
        assert!(sel.matches("auth.login.title"));
        assert!(!sel.matches("nav.home"));
    }

    #[test]
    fn empty_matches_all() {
        let sel = KeySelector::from_specs(&[]).unwrap();
        assert!(sel.matches("anything"));
        assert!(sel.is_any());
    }

    #[test]
    fn merge_unions() {
        let a = KeySelector::from_specs(&["a.*".to_string()]).unwrap();
        let b = KeySelector::from_specs(&["b.*".to_string()]).unwrap();
        let m = a.merge(b).unwrap();
        assert!(m.matches("a.x"));
        assert!(m.matches("b.x"));
        assert!(!m.matches("c.x"));
    }
}
