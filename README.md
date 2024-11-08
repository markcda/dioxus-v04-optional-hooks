# dioxus-v04-optional-hooks

Simplifies future hooks that should be reusable.

## Usage

```rust,ignore
use dioxus_v04_optional_hooks::{FutureHook, StartupGuard};
use dioxus::prelude::*;

...

let project_selected = use_state(cx, || 0);
let get_project_card_fut = FutureHook::new(cx, StartupGuard::Enable, (project_selected,), |(project_selected,)| async move {
    get_project_info(*project_selected).await
});

...

cx.render({
    get_project_card_fut.lazy_fetch();
    if let Some(project_card) = get_project_card_fut.read_clone(false) {
        ...
    }
})
```
