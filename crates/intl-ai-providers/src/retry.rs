//! Shared retry policy: every error class is retried EXCEPT auth
//! failures (a bad key never succeeds), identical for HTTP and command
//! transports. Rate limits back off exponentially and honor a
//! server-supplied Retry-After hint when present (plan 5.1.10/M9).

use intl_ai_core::error::{Error, ErrorType, Result};
use std::time::Duration;

/// Exponential backoff for rate limits: 250ms << attempt, capped. A
/// server Retry-After overrides the computed delay upward.
const BACKOFF_BASE: Duration = Duration::from_millis(250);
const BACKOFF_CAP: Duration = Duration::from_secs(30);

/// Runs `attempt` up to `max_retries` times, returning the last error.
/// The attempt includes fetching AND parsing — parse errors consume
/// retry budget too (TS parity: every class retried).
pub fn attempt_with_retries<F, T>(max_retries: u32, mut attempt: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let tries = max_retries.max(1);
    let mut last: Option<Error> = None;
    for i in 0..tries {
        match attempt() {
            Ok(out) => return Ok(out),
            Err(e) => {
                let is_last = i + 1 == tries;
                eprintln!(
                    "intl-ai: attempt {}/{} failed ({}): {}",
                    i + 1,
                    tries,
                    kind_name(&e),
                    e
                );
                if is_last || !e.retryable() {
                    return Err(e);
                }
                let delay = retry_delay(i, &e);
                if !delay.is_zero() {
                    std::thread::sleep(delay);
                }
                last = Some(e);
            }
        }
    }
    Err(last.unwrap_or_else(|| Error::transport(ErrorType::Unknown, "no attempts")))
}

/// Only rate limits sleep between attempts: exponential backoff, raised
/// to the server's Retry-After when it asks for longer. Every other
/// error class retries immediately (unchanged W1 policy).
fn retry_delay(attempt_index: u32, e: &Error) -> Duration {
    match e {
        Error::Transport {
            kind: ErrorType::RateLimit,
            ..
        } => {
            let backoff = BACKOFF_BASE
                .saturating_mul(1u32 << attempt_index.min(7))
                .min(BACKOFF_CAP);
            e.retry_after().unwrap_or_default().max(backoff)
        }
        _ => Duration::ZERO,
    }
}

fn kind_name(e: &Error) -> &'static str {
    match e {
        Error::Transport { kind, .. } => match kind {
            ErrorType::RateLimit => "rate_limit",
            ErrorType::Http => "http",
            ErrorType::SpawnFailure => "spawn_failure",
            ErrorType::Timeout => "timeout",
            ErrorType::ProcessExit => "process_exit",
            ErrorType::ParseError => "parse_error",
            ErrorType::OutputTruncated => "output_truncated",
            ErrorType::Validation => "validation",
            ErrorType::Empty => "empty",
            ErrorType::Config => "config",
            ErrorType::Lockfile => "lockfile",
            ErrorType::Io => "io",
            ErrorType::Auth => "auth",
            ErrorType::Unknown => "unknown",
        },
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn retries_all_classes_then_succeeds() {
        let calls = Mutex::new(0);
        let out = attempt_with_retries(3, || {
            *calls.lock().unwrap() += 1;
            if *calls.lock().unwrap() < 3 {
                Err(Error::transport(ErrorType::RateLimit, "429"))
            } else {
                Ok("done".to_string())
            }
        })
        .unwrap();
        assert_eq!(out, "done");
        assert_eq!(*calls.lock().unwrap(), 3);
    }

    #[test]
    fn exhausted_returns_last() {
        let calls = Mutex::new(0);
        let err = attempt_with_retries(2, || {
            *calls.lock().unwrap() += 1;
            Err::<String, _>(Error::transport(ErrorType::Timeout, "t"))
        })
        .unwrap_err();
        assert!(matches!(
            err,
            Error::Transport {
                kind: ErrorType::Timeout,
                ..
            }
        ));
        assert_eq!(*calls.lock().unwrap(), 2);
    }

    #[test]
    fn zero_retries_means_one_attempt() {
        let calls = Mutex::new(0);
        let _ = attempt_with_retries(0, || {
            *calls.lock().unwrap() += 1;
            Err::<String, _>(Error::transport(ErrorType::Http, "x"))
        });
        assert_eq!(*calls.lock().unwrap(), 1);
    }

    #[test]
    fn auth_errors_are_not_retried() {
        let calls = Mutex::new(0);
        let err = attempt_with_retries(5, || {
            *calls.lock().unwrap() += 1;
            Err::<String, _>(Error::transport(ErrorType::Auth, "401 bad key"))
        })
        .unwrap_err();
        assert!(matches!(
            err,
            Error::Transport {
                kind: ErrorType::Auth,
                ..
            }
        ));
        assert_eq!(*calls.lock().unwrap(), 1);
    }

    #[test]
    fn retry_after_hint_raises_the_delay() {
        let e =
            Error::transport(ErrorType::RateLimit, "429").with_retry_after(Duration::from_secs(5));
        assert_eq!(retry_delay(0, &e), Duration::from_secs(5));

        // Backoff grows: attempt 2 -> 1s floor.
        let e = Error::transport(ErrorType::RateLimit, "429");
        assert_eq!(retry_delay(2, &e), Duration::from_secs(1));

        // Non-rate-limit errors never sleep.
        let e = Error::transport(ErrorType::Http, "500");
        assert_eq!(retry_delay(0, &e), Duration::ZERO);
    }
}
