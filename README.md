# `sui`

> the signals ui library
> made for my game [signals](https://github.com/manen/signals), i separated it into its own repository to make it easier to use for other projects.

`sui` is a small but flexible ui component library built for [`raylib`](https://github.com/manen/raylib-rs), written in Rust.

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
