//! Scoped, clock-free pack loader instrumentation. The caller owns timing and
//! presentation; ordinary loaders and terminal clients remain silent.
use std::cell::Cell;

type Observer = fn(&'static str, bool);
thread_local! {
    static OBSERVER: Cell<Option<Observer>> = const { Cell::new(None) };
}

/// Observe nested loader stages only for this thread and closure. `true` begins
/// a stage; `false` ends it, including when validation returns an error.
/// The previous observer is restored on return or unwinding.
pub fn with_compiled_pack_load_observer<T>(observer: Observer, load: impl FnOnce() -> T) -> T {
    struct Reset(Option<Observer>);
    impl Drop for Reset {
        fn drop(&mut self) {
            OBSERVER.set(self.0);
        }
    }
    let _reset = Reset(OBSERVER.replace(Some(observer)));
    load()
}

pub(crate) struct Span {
    name: &'static str,
    observer: Option<Observer>,
}
impl Span {
    pub(crate) fn new(name: &'static str) -> Self {
        // Copy the callback before invocation: observers may reenter the loader
        // or install a nested observer without borrowing thread-local storage.
        let observer = OBSERVER.get();
        if let Some(observer) = observer {
            observer(name, true);
        }
        Self { name, observer }
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        if let Some(observer) = self.observer {
            observer(self.name, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    thread_local! { static EVENTS: RefCell<Vec<(&'static str, bool)>> = const { RefCell::new(Vec::new()) }; }
    fn record(name: &'static str, begin: bool) {
        EVENTS.with_borrow_mut(|v| v.push((name, begin)));
    }
    fn nested(name: &'static str, begin: bool) {
        record(name, begin);
        if begin && name == "outer" {
            with_compiled_pack_load_observer(record, || {
                let _s = Span::new("reentrant");
            });
        }
    }
    #[test]
    fn observer_is_scoped_balanced_on_error_and_reentrant() {
        EVENTS.with_borrow_mut(Vec::clear);
        let error: Result<(), ()> = with_compiled_pack_load_observer(nested, || {
            let _outer = Span::new("outer");
            let _inner = Span::new("inner");
            Err(())
        });
        assert!(error.is_err());
        {
            let _unobserved = Span::new("after");
        }
        assert_eq!(
            EVENTS.with_borrow(Clone::clone),
            vec![
                ("outer", true),
                ("reentrant", true),
                ("reentrant", false),
                ("inner", true),
                ("inner", false),
                ("outer", false)
            ]
        );
        assert!(OBSERVER.get().is_none());
    }
    #[test]
    fn observer_reset_survives_unwinding() {
        EVENTS.with_borrow_mut(Vec::clear);
        let result = std::panic::catch_unwind(|| {
            with_compiled_pack_load_observer(record, || {
                let _stage = Span::new("failure");
                panic!("test loader failure");
            })
        });
        assert!(result.is_err());
        assert!(OBSERVER.get().is_none());
        assert_eq!(
            EVENTS.with_borrow(Clone::clone),
            vec![("failure", true), ("failure", false)]
        );
    }
}
