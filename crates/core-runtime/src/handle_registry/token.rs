use std::num::NonZeroU64;
use std::sync::Mutex;

/// Serializes allocation so each successful Core token is nonzero and never reused.
#[allow(dead_code)] // The registry starts consuming this issuer in the next implementation slice.
pub(crate) struct TokenIssuer {
    next: Mutex<Option<NonZeroU64>>,
}

#[allow(dead_code)]
impl TokenIssuer {
    const fn new() -> Self {
        Self {
            next: Mutex::new(NonZeroU64::new(1)),
        }
    }

    fn with_next(next: Option<NonZeroU64>) -> Self {
        Self {
            next: Mutex::new(next),
        }
    }

    /// Returns `None` when the token space is exhausted or synchronization is poisoned.
    pub(super) fn issue(&self) -> Option<NonZeroU64> {
        let mut next = self.next.lock().ok()?;
        let token = (*next)?;
        *next = token.get().checked_add(1).and_then(NonZeroU64::new);
        Some(token)
    }
}

#[allow(dead_code)] // One static issuer is shared by all Runtime instances in this Core process.
pub(super) static PROCESS_TOKEN_ISSUER: TokenIssuer = TokenIssuer::new();

#[cfg(test)]
mod tests {
    use super::TokenIssuer;
    use std::num::NonZeroU64;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn issues_nonzero_monotonic_tokens_without_reuse() {
        let issuer = TokenIssuer::new();

        let first = issuer.issue().expect("first token");
        let second = issuer.issue().expect("second token");

        assert!(first.get() > 0);
        assert!(second > first);
    }

    #[test]
    fn issues_maximum_token_once_then_stays_exhausted() {
        let issuer = TokenIssuer::with_next(NonZeroU64::new(u64::MAX));

        assert_eq!(issuer.issue().map(NonZeroU64::get), Some(u64::MAX));
        assert_eq!(issuer.issue(), None);
        assert_eq!(issuer.issue(), None);
    }

    #[test]
    fn concurrent_issuance_returns_unique_tokens() {
        const THREADS: usize = 8;
        const TOKENS_PER_THREAD: usize = 256;
        let issuer = Arc::new(TokenIssuer::new());

        let batches = thread::scope(|scope| {
            (0..THREADS)
                .map(|_| {
                    let issuer = Arc::clone(&issuer);
                    scope.spawn(move || {
                        (0..TOKENS_PER_THREAD)
                            .map(|_| issuer.issue().expect("token available").get())
                            .collect::<Vec<_>>()
                    })
                })
                .map(|task| task.join().expect("issuer worker completes"))
                .collect::<Vec<_>>()
        });

        let mut tokens = batches.into_iter().flatten().collect::<Vec<_>>();
        tokens.sort_unstable();
        assert_eq!(tokens.len(), THREADS * TOKENS_PER_THREAD);
        assert!(tokens.iter().all(|token| *token > 0));
        assert!(tokens.windows(2).all(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn poisoned_synchronization_fails_closed() {
        let issuer = Arc::new(TokenIssuer::new());
        let poisoned = Arc::clone(&issuer);
        let worker = thread::spawn(move || {
            let _guard = poisoned.next.lock().expect("mutex initially healthy");
            panic!("poison the issuer mutex for this test");
        });

        assert!(worker.join().is_err());
        assert_eq!(issuer.issue(), None);
    }
}
