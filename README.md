# Proof of concept

This code is proof of concept code for:
https://github.com/RReverser/serde-xml-rs/issues/55

The code will read the following example file:

```xml
<root>
    <field1>abc</field1>
    <field1>def</field1>

    <field2>lmn</field2>
    <field2>opq</field2>

    <field1>ghi</field1>

    <field2>again 2</field2>

    <unknown_tag>this should be in the unknown tag</unknown_tag>
</root>
```

The crate `serde-xml-rs` (tested with version 0.4.0) with `serde` (v1.0.106)
will fail at this with the default implementation because there are duplicate
tags.

It will fail with:
```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value:
Custom { field: "duplicate field `field1`" }
```

This is because: `<field1>ghi</field1>` occurs after a `<field2>`-tag.

In this proof of concept we implement our own `Deserialize` implementation.
We use this implementation to create derive macro.
This way we can implement this using `#[derive(DeserializeBestEffort)]`.
Because of some limitations there are some other traits like `Default` needed.

## Behavior
The code will try to fail as few times as possible.
So if no value is found it will use the `Default` trait to fill the value.
For values that are not lists like `String` it will replace the value with the
last value in the it finds in the file.
For lists it will append the value to the list.
If it finds a value that does not have a key defined in the `struct` it will
add it to the `unknown` variable.

This behavior is separate from the proof of concept. And other behavior
could be implemented with small modifications.

In the `#[derive(DeserializeBestEffort)]` case I also supported the
`#[serde(alias = "name")]` macro. Other
[attributes](https://serde.rs/attributes.html) are not implemented.

## Reason for this proof of concept
I hope this helps some people and that is could be implemented into
`serde-xml-rs` and or `serde`. So that this behavior will be in the crates.
Although the code is not incredibly written and might look weird is some cases.
This was my first proper use of the `proc_macro_derive` macro.
If you have any question let me know at: contact.ralph.b at gmail.com
This code will NOT be maintained or updated. (Date of release: 2020/04/21)

## Execute
Just run `$ cargo run` in the main folder.

Expected output:
```
Print Parsed output: RootWorking {
    field1: [
        "abc",
        "def",
        "ghi",
    ],
    field2: [
        "lmn",
        "opq",
        "again 2",
    ],
    unknown: {
        "unknown_tag": Object(
            {
                "$value": String(
                    "this should be in the unknown tag",
                ),
            },
        ),
    },
}
```

## License
It is licensed under the MIT License.
