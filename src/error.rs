use crate::alloc::borrow::ToOwned;
use crate::tree::{denormalize_params, Node};
use alloc::string::String;

use core::fmt;

/// Represents errors that can occur when inserting a new route.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InsertError {
    /// Attempted to insert a path that conflicts with an existing route.
    Conflict {
        /// The existing route that the insertion is conflicting with.
        with: String,
    },
    /// Only one parameter per route segment is allowed.
    TooManyParams,
    /// Parameters must be registered with a name.
    UnnamedParam,
    /// Catch-all parameters are only allowed at the end of a path.
    InvalidCatchAll,
}

impl fmt::Display for InsertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conflict { with } => {
                write!(
                    f,
                    "insertion failed due to conflict with previously registered route: {}",
                    with
                )
            }
            Self::TooManyParams => write!(f, "only one parameter is allowed per path segment"),
            Self::UnnamedParam => write!(f, "parameters must be registered with a name"),
            Self::InvalidCatchAll => write!(
                f,
                "catch-all parameters are only allowed at the end of a route"
            ),
        }
    }
}

impl InsertError {
    pub(crate) fn conflict<T>(route: &[u8], prefix: &[u8], current: &Node<T>) -> Self {
        // The new route would have had to replace the current node in the tree.
        if prefix == current.prefix {
            let mut route = route.to_owned();
            denormalize_params(&mut route, &current.param_remapping);
            return InsertError::Conflict {
                with: String::from_utf8(route).unwrap(),
            };
        }

        let mut route = route[..route.len() - prefix.len()].to_owned();

        if !route.ends_with(&current.prefix) {
            route.extend_from_slice(&current.prefix);
        }

        let mut last = current;
        while let Some(node) = last.children.first() {
            last = node;
        }

        let mut current = current.children.first();
        while let Some(node) = current {
            route.extend_from_slice(&node.prefix);
            current = node.children.first();
        }

        denormalize_params(&mut route, &last.param_remapping);

        InsertError::Conflict {
            with: String::from_utf8(route).unwrap(),
        }
    }
}

/// A failed match attempt.
///
/// ```
/// use matchit::{MatchError, Router};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut router = Router::new();
/// router.insert("/home", "Welcome!")?;
/// router.insert("/blog/", "Our blog.")?;
///
/// // a route exists without the trailing slash
/// if let Err(err) = router.at("/home/") {
///     assert_eq!(err, MatchError::ExtraTrailingSlash);
/// }
///
/// // a route exists with a trailing slash
/// if let Err(err) = router.at("/blog") {
///     assert_eq!(err, MatchError::MissingTrailingSlash);
/// }
///
/// // no routes match
/// if let Err(err) = router.at("/foobar") {
///     assert_eq!(err, MatchError::NotFound);
/// }
/// # Ok(())
/// # }
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MatchError {
    /// The path was missing a trailing slash.
    MissingTrailingSlash,
    /// The path had an extra trailing slash.
    ExtraTrailingSlash,
    /// No matching route was found.
    NotFound,
}

impl MatchError {
    pub(crate) fn unsure(full_path: &[u8]) -> Self {
        if full_path[full_path.len() - 1] == b'/' {
            MatchError::ExtraTrailingSlash
        } else {
            MatchError::MissingTrailingSlash
        }
    }
}

impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            MatchError::MissingTrailingSlash => "match error: expected trailing slash",
            MatchError::ExtraTrailingSlash => "match error: found extra trailing slash",
            MatchError::NotFound => "match error: route not found",
        };

        write!(f, "{}", msg)
    }
}
