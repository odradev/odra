//! [`HostEnv::concurrently`]: the same host-side work run for many items at once, on worker
//! threads of this machine, each with an environment of its own.
//!
//! The host environment models what contracts and tests do on a backend; this module is about the
//! machine that runs them (threads, `ODRA_MAX_THREADS`), which is why it lives apart. Only a
//! backend that can be used from several threads at once (livenet) provides a
//! [`ThreadEnvFactory`]; on OdraVM and CasperVM the items run one after another.

use crate::host::HostEnv;
use crate::prelude::*;
use alloc::sync::Arc;

/// The most worker threads [`HostEnv::concurrently`] starts by default; each one keeps its own
/// node connection and caches.
const MAX_CONCURRENT_THREADS: usize = 8;

/// The environment variable that overrides [`MAX_CONCURRENT_THREADS`] for
/// [`HostEnv::concurrently`].
const MAX_THREADS_ENV: &str = "ODRA_MAX_THREADS";

/// The thread limit [`HostEnv::concurrently`] uses for the value of `ODRA_MAX_THREADS`: the
/// default when it is not set or empty (a blank entry in a `.env` file), at least one thread, and
/// a panic on a value that is not a number, so a typo does not silently fall back to the default.
fn max_threads_from_env(value: Option<&str>) -> usize {
    match value {
        None => MAX_CONCURRENT_THREADS,
        Some(value) if value.trim().is_empty() => MAX_CONCURRENT_THREADS,
        Some(value) => value.trim().parse::<usize>().unwrap_or_else(|_| {
            panic!("{MAX_THREADS_ENV} must be a number of threads, got {value:?}")
        })
    }
}

/// Builds an independent [`HostEnv`] for a worker thread of [`HostEnv::concurrently`]; see
/// [`HostContext::thread_env_factory`](crate::host::HostContext::thread_env_factory).
#[derive(Clone)]
pub struct ThreadEnvFactory(Arc<dyn Fn() -> HostEnv + Send + Sync>);

impl ThreadEnvFactory {
    /// Wraps the function that builds the environment; it is called once per worker thread.
    pub fn new(build: impl Fn() -> HostEnv + Send + Sync + 'static) -> Self {
        Self(Arc::new(build))
    }

    /// Builds a new environment.
    pub fn build(&self) -> HostEnv {
        (self.0)()
    }
}

impl HostEnv {
    /// Runs `f` once per item and returns the results in the order of the items.
    ///
    /// On livenet the items are spread over a few worker threads, each with its own node
    /// connection and its own `HostEnv` (same caller and gas as this one), so deploys and reads
    /// that do not depend on each other overlap instead of waiting for one another. On OdraVM and
    /// CasperVM the items run one after another on this thread, with this environment.
    ///
    /// `f` gets the environment to use; it must not capture this one (the compiler enforces it:
    /// `HostEnv` cannot be sent to another thread). Deploy in the closure and hand back the
    /// address, then `load` it in the caller's environment:
    ///
    /// ```ignore
    /// env.set_gas(cspr!(450));
    /// let addresses = env.concurrently(vec![args_a, args_b, args_c], |env, args| {
    ///     Erc20::deploy(env, args).address()
    /// });
    /// let tokens: Vec<Erc20HostRef> = addresses.iter().map(|a| Erc20::load(&env, *a)).collect();
    /// let supplies = env.concurrently(addresses, |env, address| {
    ///     Erc20::load(env, address).total_supply()
    /// });
    /// ```
    ///
    /// At most 8 worker threads run at once; the `ODRA_MAX_THREADS` environment variable changes
    /// that, and [`concurrently_with`](Self::concurrently_with) takes the limit as an argument.
    ///
    /// # Panics
    ///
    /// A panic in `f` (a failed `deploy`, a revert) is propagated after the other items finish.
    /// Panics if `ODRA_MAX_THREADS` is set to something that is not a number.
    pub fn concurrently<T, R, F>(&self, items: Vec<T>, f: F) -> Vec<R>
    where
        T: Send,
        R: Send,
        F: Fn(&HostEnv, T) -> R + Sync
    {
        let max_threads = max_threads_from_env(std::env::var(MAX_THREADS_ENV).ok().as_deref());
        self.concurrently_with(max_threads, items, f)
    }

    /// [`concurrently`](Self::concurrently) with at most `max_threads` worker threads, regardless
    /// of `ODRA_MAX_THREADS`.
    ///
    /// A lower limit spares a node that rate-limits connections, a higher one overlaps more
    /// transactions. `0` is treated as `1`. On OdraVM and CasperVM the items run one after another
    /// on this thread whatever the limit.
    pub fn concurrently_with<T, R, F>(&self, max_threads: usize, items: Vec<T>, f: F) -> Vec<R>
    where
        T: Send,
        R: Send,
        F: Fn(&HostEnv, T) -> R + Sync
    {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Mutex;

        let Some(factory) = self.backend.thread_env_factory() else {
            return items.into_iter().map(|item| f(self, item)).collect();
        };

        let workers = items.len().min(max_threads.max(1));
        let items: Vec<Mutex<Option<T>>> = items.into_iter().map(|i| Mutex::new(Some(i))).collect();
        let results: Vec<Mutex<Option<R>>> = items.iter().map(|_| Mutex::new(None)).collect();
        let next = AtomicUsize::new(0);

        std::thread::scope(|scope| {
            for _ in 0..workers {
                scope.spawn(|| {
                    let env = factory.build();
                    loop {
                        let index = next.fetch_add(1, Ordering::SeqCst);
                        let Some(slot) = items.get(index) else { break };
                        let item = slot
                            .lock()
                            .expect("item slot poisoned")
                            .take()
                            .expect("every item is taken exactly once");
                        let result = f(&env, item);
                        *results[index].lock().expect("result slot poisoned") = Some(result);
                    }
                });
            }
        });

        results
            .into_iter()
            .map(|slot| {
                slot.into_inner()
                    .expect("result slot poisoned")
                    .expect("every item produced a result")
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::MockHostContext;
    use casper_types::account::AccountHash;

    #[test]
    fn concurrently_runs_on_the_calling_thread_without_a_factory() {
        let mut ctx = MockHostContext::new();
        ctx.expect_thread_env_factory().returning(|| None);
        let env = HostEnv::new(Rc::new(ctx));
        let main_thread = std::thread::current().id();

        let results = env.concurrently((0..5).collect(), |_, i: u32| {
            (i * 2, std::thread::current().id())
        });

        assert_eq!(
            results,
            (0..5).map(|i| (i * 2, main_thread)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn concurrently_spreads_items_over_worker_threads_and_keeps_the_order() {
        let mut ctx = MockHostContext::new();
        ctx.expect_thread_env_factory().returning(|| {
            Some(ThreadEnvFactory::new(|| {
                let mut ctx = MockHostContext::new();
                ctx.expect_get_account()
                    .returning(|_| Address::Account(AccountHash::new([7; 32])));
                HostEnv::new(Rc::new(ctx))
            }))
        });
        let env = HostEnv::new(Rc::new(ctx));
        let main_thread = std::thread::current().id();

        let results = env.concurrently((0..50u32).collect(), |env, i| {
            // The worker's own environment is usable, and the work really runs elsewhere.
            assert_eq!(
                env.get_account(0),
                Address::Account(AccountHash::new([7; 32]))
            );
            std::thread::sleep(core::time::Duration::from_millis(1));
            (i * 2, std::thread::current().id())
        });

        let values: Vec<u32> = results.iter().map(|(v, _)| *v).collect();
        assert_eq!(values, (0..50).map(|i| i * 2).collect::<Vec<_>>());
        let threads: std::collections::HashSet<_> = results.iter().map(|(_, t)| *t).collect();
        assert!(threads.len() > 1, "expected several worker threads");
        assert!(!threads.contains(&main_thread));
    }

    fn env_with_thread_factory() -> HostEnv {
        let mut ctx = MockHostContext::new();
        ctx.expect_thread_env_factory().returning(|| {
            Some(ThreadEnvFactory::new(|| {
                HostEnv::new(Rc::new(MockHostContext::new()))
            }))
        });
        HostEnv::new(Rc::new(ctx))
    }

    #[test]
    fn concurrently_with_one_thread_runs_every_item_on_a_single_worker_in_order() {
        let env = env_with_thread_factory();
        let main_thread = std::thread::current().id();

        let results = env.concurrently_with(1, (0..20u32).collect(), |_, i| {
            (i * 3, std::thread::current().id())
        });

        let values: Vec<u32> = results.iter().map(|(v, _)| *v).collect();
        assert_eq!(values, (0..20).map(|i| i * 3).collect::<Vec<_>>());
        let threads: std::collections::HashSet<_> = results.iter().map(|(_, t)| *t).collect();
        assert_eq!(threads.len(), 1);
        assert!(!threads.contains(&main_thread));
    }

    #[test]
    fn concurrently_with_zero_threads_is_treated_as_one() {
        let env = env_with_thread_factory();

        let results = env.concurrently_with(0, (0..10u32).collect(), |_, i| {
            (i + 1, std::thread::current().id())
        });

        let values: Vec<u32> = results.iter().map(|(v, _)| *v).collect();
        assert_eq!(values, (1..=10).collect::<Vec<_>>());
        let threads: std::collections::HashSet<_> = results.iter().map(|(_, t)| *t).collect();
        assert_eq!(threads.len(), 1);
    }

    #[test]
    fn concurrently_with_never_starts_more_threads_than_the_limit() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let env = env_with_thread_factory();
        let running = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);

        let results = env.concurrently_with(3, (0..30u32).collect(), |_, i| {
            let now = running.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(now, Ordering::SeqCst);
            std::thread::sleep(core::time::Duration::from_millis(2));
            running.fetch_sub(1, Ordering::SeqCst);
            i
        });

        assert_eq!(results, (0..30).collect::<Vec<_>>());
        assert!(peak.load(Ordering::SeqCst) <= 3);
    }

    #[test]
    fn concurrently_with_ignores_the_limit_without_a_factory() {
        let mut ctx = MockHostContext::new();
        ctx.expect_thread_env_factory().returning(|| None);
        let env = HostEnv::new(Rc::new(ctx));
        let main_thread = std::thread::current().id();

        let results =
            env.concurrently_with(4, vec![1u8, 2, 3], |_, i| (i, std::thread::current().id()));

        assert_eq!(
            results,
            vec![(1, main_thread), (2, main_thread), (3, main_thread)]
        );
    }

    #[test]
    fn max_threads_comes_from_the_environment_variable_or_defaults_to_eight() {
        assert_eq!(max_threads_from_env(None), 8);
        assert_eq!(max_threads_from_env(Some("")), 8);
        assert_eq!(max_threads_from_env(Some(" ")), 8);
        assert_eq!(max_threads_from_env(Some("2")), 2);
        assert_eq!(max_threads_from_env(Some(" 16 ")), 16);
        // `concurrently_with` treats it as one.
        assert_eq!(max_threads_from_env(Some("0")), 0);
    }

    #[test]
    #[should_panic(expected = "ODRA_MAX_THREADS must be a number of threads")]
    fn max_threads_rejects_a_value_that_is_not_a_number() {
        max_threads_from_env(Some("eight"));
    }
}
