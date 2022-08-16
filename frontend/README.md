# Duck Chess Frontend

So, you like web development. This uses Yew and compiles to Wasm/html which the
backend serves. You'll need to `cargo install trunk` and run that to
build/package everything (don't use `trunk serve`, the backend should serve the
frontend for CORS reasons).

This repo uses Yew in the Elm style. Yew also supports a reactive style which
might be worth porting to at some point just to see if it's more ergonomic.

Pretty much every file in here follows the same pattern: define a Msg
struct, a Props struct, a Model struct, and impl Component for the Model.
