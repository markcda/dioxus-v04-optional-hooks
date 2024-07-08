use std::fmt::Debug;
use std::future::Future;
use dioxus::prelude::*;

/// Optional future hook.
#[derive(Copy, Clone)]
pub struct FutureHook<'a, T, E>
  where
    T: 'static + ?Sized + Clone,
    E: 'static + ?Sized + Clone + Debug,
{
  future: &'a UseFuture<Result<T, E>>,
  outdated_marker: &'a UseState<bool>,
}

#[derive(PartialEq, Eq)]
pub enum FutureState {
  Empty,
  Ready,
  Error,
  Outdated,
  Reloading,
}

#[derive(PartialEq, Eq)]
pub enum StartupGuard {
  Disable = 0,
  Enable = 1,
}

impl<'a, T, E> FutureHook<'a, T, E>
  where
    T: 'static + Clone,
    E: 'static + Clone + Debug,
{
  /// Creates new optional future.
  ///
  /// Example:
  /// ```rust
  /// use dioxus_v04_optional_hooks::{FutureHook, StartupGuard};
  /// ...
  /// let generate_fut = FutureHook::new(cx, StartupGuard::Enable, (dependency_state_hook,) |(dependency_state_hook,)| {
  ///   async move {
  ///     some_func(*dependency_state_hook).await
  ///   }
  /// });
  /// ```
  pub fn new<
    D: UseFutureDep,
    F: Future<Output = Result<T, E>> + 'static
  >(
    cx: Scope<'a>,
    startup_guard: StartupGuard,
    dependencies: D,
    fut: impl FnOnce(D::Out) -> F
  ) -> Self {
    let outdated = startup_guard == StartupGuard::Enable;
    Self {
      future: use_future(cx, dependencies, fut),
      outdated_marker: use_state(cx, || outdated),
    }
  }

  /// Extends the standard future states by adding one more.
  pub fn check_state(&self) -> FutureState {
    let val = match self.future.state() {
      UseFutureState::Pending => FutureState::Empty,
      UseFutureState::Complete(Ok(_)) => FutureState::Ready,
      UseFutureState::Complete(Err(_)) => FutureState::Error,
      UseFutureState::Reloading(_) => FutureState::Reloading,
    };
    if (val == FutureState::Ready || val == FutureState::Error) && self.is_outdated() {
      return FutureState::Outdated
    }
    val
  }

  /// Reads the future value, if any.
  pub fn read(&self, allow_cache_while_reloading: bool) -> Option<&'a T> {
    if self.is_outdated() { return None }
    if !allow_cache_while_reloading {
      match self.check_state() {
        FutureState::Empty | FutureState::Reloading | FutureState::Error | FutureState::Outdated => { None },
        FutureState::Ready => {
          let val = self.future.value().as_ref().unwrap().as_ref().unwrap();
          Some(val)
        },
      }
    } else {
      match self.check_state() {
        FutureState::Empty | FutureState::Error => { None },
        FutureState::Ready | FutureState::Reloading => {
          let val_p = self.future.value();
          let val = val_p.as_ref().unwrap();
          match val.as_ref() {
            Err(_) => None,
            Ok(val) => Some(val),
          }
        },
        FutureState::Outdated => {
          if let UseFutureState::Complete(Ok(val)) = self.future.state() {
            Some(val)
          } else {
            None
          }
        },
      }
    }
  }

  /// Clones the value.
  pub fn read_clone(&self, allow_cache_while_reloading: bool) -> Option<T> {
    self.read(allow_cache_while_reloading).cloned()
  }

  /// Gets the future value directly.
  pub fn read_unchecked(&self) -> Option<&'a Result<T, E>> {
    self.future.value()
  }

  /// Restarts the future hook.
  pub fn restart(&self) {
    if self.check_state() == FutureState::Empty || self.check_state() == FutureState::Reloading { return }

    self.outdated_marker.set(false);
    self.future.restart();
  }

  /// Restarts the future only if it's outdated.
  pub fn fetch(&self) {
    if self.is_outdated() { self.restart(); }
  }

  /// Checks if the future is outdated.
  pub fn is_outdated(&self) -> bool {
    **self.outdated_marker
  }

  /// Sets the future outdated.
  pub fn set_outdated(&self) {
    self.outdated_marker.set(true)
  }

  /// Clones outdated marker to use into closures.
  pub fn get_outdated_marker(&self) -> &'a UseState<bool> {
    &self.outdated_marker
  }
}