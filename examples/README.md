### Usage examples for zephyrus.

This directory contains several examples on how to use the ``zephyrus`` crate.

The examples can be run by using the following cargo command:
```
cargo run --example <example name here>
```

*Please note that most of the examples are made without spawning dedicated tasks, this is to simplify the examples
and to avoid having to always have to wrap the framework inside an `Arc`. However, it is preferred to process each
interaction inside a dedicated task to avoid having to wait until a command finishes its execution to execute the next
one*

To add an example to this directory, feel free to open a Pull request.


