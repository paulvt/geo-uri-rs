# geo-uri-rs

A Rust crate (`geo-uri`) for parsing uniform resource identifiers for
geographic locations (geo URIs) according to
IEEE RFC [5870](https://www.rfc-editor.org/rfc/rfc5870).
This crate allows for parsing and generating geo URIs in the correct format.
It's parser is currently somewhat more liberal than the proposed standard.

It supports geolocations specified by latitude and longitude, but also
optionally altitude and an uncertainty radius.
The only supported coordinate reference system is
[WGS-84](https://en.wikipedia.org/wiki/World_Geodetic_System#WGS84).

## Usage

Just run the following to add this library to your project:

```sh
$ cargo add geo-uri
    Updating crates.io index
      Adding thiserror v??? to dependencies.
```

### Parsing

Use either the [`FromStr`](std::str::FromStr) or
[`TryFrom`](std::convert::TryFrom) traits to parse a geo URI string:

```rust
use geo_uri::GeoUri;

let geo_uri = GeoUri::try_from("geo:52.107,5.134,3.6;u=1000");
assert!(geo_uri.is_ok());

let geo_uri2 = GeoUri::from_str("geo:52.107,5.134;u=2000");
assert!(geo_uri2.is_ok());
```

It is also possible to call the parse function directly:

```rust
use geo_uri::GeoUri;

let geo_uri3 = GeoUri::parse("geo:52.107,5.134,3.6;u=1000");
assert!(geo_uri3.is_ok());
```

### Generating

Use either the [`ToString`](std::string::ToString) or
[`Display`](std::fmt::Display) trait to generate an geo URI after building it:

```rust
use geo_uri::GeoUri;

let geo_uri = GeoUri::builder()
                  .latitude(52.107)
                  .longitude(5.134)
                  .uncertainty(1_000)
                  .build();
assert_eq!(
    geo_uri.to_string(),
    String::from("geo:52.107,5.134;u=1000")
);
assert_eq!(
    format!("{}", geo_uri),
    String::from("geo:52.107,5.134;u=1000")
);
```

## License

geo-uri-rs is licensed under the MIT license (see the `LICENSE` file or
<http://opensource.org/licenses/MIT>).
