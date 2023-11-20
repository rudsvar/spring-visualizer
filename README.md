# Spring Visualizer

Spring visualizer produces a graph of the application context based on your Java code.

## Features

1. Configuration classes with
   - edges to their imports and
   - bean definitions.
2. Component scanning overview including
   - which packages and
   - which component are scanned,
   - with warnings when components are not scanned. (TODO)
3. Overview of
   - autowired dependencies of components, and
   - warnings when they are not component-scanned components or defined as beans. (TODO)
4. Configuration of what to include in the final graph with clap.

## Example

If you run the command below

```
RUST_LOG=spring_visualizer=debug cargo run -- com/example/demo > demo/example.dot && dot -Tpng demo/example.dot -o demo/example.png
```

then you will get the following output:

![](./demo/example.png)
