##Example Circuit Definition File

```
attributes {
    country: 48
    age: 21
}
```

It will be converted into the following Rust structure to be used by the circuit generator:

```
pub struct UserAttributes {
    pub country_code: Option<u16>,
    pub age: Option<u8>
}
```
