# Spring Visualizer

Spring visualizer produces a graph of the application context based on your Java code.

## Features

1. Configuration classes with
   [x] edges to their imports and
   [x] bean definitions.
2. Component scanning overview including
   [x] which packages and
   [x] which component are scanned,
   [ ] with warnings when components are not scanned.
3. Overview of
   [x] autowired dependencies of components, and
   [ ] warnings when they are not component-scanned components or defined as beans.

## Example

If you run the command below

```
cargo run -- com/example/demo > example.dot && dot -Tpng example.dot -o example.png
```

then you will get the following output:

![](./example.png)

## TODO

- Configuration of what to include in the final graph with clap
