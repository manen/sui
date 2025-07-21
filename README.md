# `sui`

> the ui library made for my game [signals](https://github.com/manen/signals) \
> i separated it into its own repository to make it easier to use for other projects.

`sui` is a small but (somewhat) flexible ui component library built for [`raylib`](https://github.com/manen/raylib-rs), written in Rust.

at the core, it's just one trait:

```rs
pub trait Layable {
  fn size(&self) -> (i32, i32);
  fn render(&self, d: &mut Handle, det: Details, scale: f32);

  /// this function is called by the parent of this component
  /// return events to be bubbled back
  fn pass_event(
    &self,
    event: Event,
    det: Details,
    scale: f32,
  ) -> Option<crate::core::ReturnEvent>;
}
```

with this, you can build surprisingly complex ui's and the `pass_event` function makes it possible to build whole systems as part of the ui tree.

## usage

for simplicity and developer experience, `sui` doesn't use cargo workspaces as nested workspaces don't exist in Rust (for some reason).

to use a package you want, you can either:

```toml
[dependencies]
sui.git = "https://github.com/manen/sui"
sui_runner.git = "https://github.com/manen/sui"
# etc etc
```

or if you'd like to develop sui along your game, use git submodules and include the folders in this repository in your own workspace.
