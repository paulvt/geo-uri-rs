# geo-uri-rs

A Rust crate for uniform resource identifiers for geographic locations (geo
URIs) according to IEEE [RFC 5870](https://www.rfc-editor.org/rfc/rfc5870).
This crate supports parsing and generating geo URIs in the correct format.
Its parser is currently somewhat more liberal than the proposed standard.

It supports geolocations specified by latitude and longitude, but also
optionally altitude and an uncertainty radius.
The currently only supported coordinate reference system is
[WGS-84](https://en.wikipedia.org/wiki/World_Geodetic_System#WGS84).

## Usage

Just run the following to add this library to your project:

```sh
$ cargo add geo-uri
    Updating crates.io index
      Adding geo-uri vX.Y.Z to dependencies.
```

### Parsing

Use either the [`TryFrom`](std::convert::TryFrom) trait or the
[`parse`](str::parse) method on strings to parse a geo URI string into a
[`GeoUri`] struct:

```rust
use geo_uri::GeoUri;

let geo_uri = GeoUri::try_from("geo:52.107,5.134,3.6;u=1000").expect("valid geo URI");
assert_eq!(geo_uri.latitude(), 52.107);
assert_eq!(geo_uri.longitude(), 5.134);
assert_eq!(geo_uri.altitude(), Some(3.6));
assert_eq!(geo_uri.uncertainty(), Some(1000.0));

let geo_uri: GeoUri = "geo:52.107,5.134;u=2000.0".parse().expect("valid geo URI");
assert_eq!(geo_uri.latitude(), 52.107);
assert_eq!(geo_uri.longitude(), 5.134);
assert_eq!(geo_uri.altitude(), None);
assert_eq!(geo_uri.uncertainty(), Some(2000.0));
```

It is also possible to call the parse function directly:

```rust
use geo_uri::GeoUri;

let geo_uri = GeoUri::parse("geo:52.107,5.134,3.6").expect("valid geo URI");
assert_eq!(geo_uri.latitude(), 52.107);
assert_eq!(geo_uri.longitude(), 5.134);
assert_eq!(geo_uri.altitude(), Some(3.6));
assert_eq!(geo_uri.uncertainty(), None);
```

### Generating

Use the [`GeoUriBuilder`] to construct a [`GeoUri`] struct.
Then, use either the [`ToString`](std::string::ToString) or
[`Display`](std::fmt::Display) trait to generate a geo URI string:

```rust
use geo_uri::GeoUri;

let geo_uri = GeoUri::builder()
    .latitude(52.107)
    .longitude(5.134)
    .uncertainty(1_000.0)
    .build()
    .unwrap();

assert_eq!(
    geo_uri.to_string(),
    String::from("geo:52.107,5.134;u=1000")
);
assert_eq!(
    format!("{geo_uri}"),
    String::from("geo:52.107,5.134;u=1000")
);
```

It is also possible to construct a [`GeoUri`] struct from coordinate tuples
using the [`TryFrom`](std::convert::TryFrom) trait:

```rust
use geo_uri::GeoUri;

let geo_uri = GeoUri::try_from((52.107, 5.134)).expect("valid coordinates");
let geo_uri = GeoUri::try_from((52.107, 5.134, 3.6)).expect("valid coordinates");
```

### Feature: `url`

You can enable the `url` feature to convert from and to
[`Url`](https://docs.rs/url/2/url/struct.Url.html) structs from the
[`url`](https://docs.rs/url/2/url) crate.

Enable the feature in your `Cargo.toml` first:

```toml
geo-uri = { version = "X.Y.Z", features = ["url"] }
```

Then you can do:

```rust
use geo_uri::GeoUri;
use url::Url;

let url = Url::parse("geo:52.107,5.134,3.6").expect("valid URL");
let geo_uri = GeoUri::try_from(&url).expect("valid geo URI");
let geo_url = Url::from(geo_uri);

assert_eq!(url, geo_url);
```

Note that it is always possible to transform a [`GeoUri`] into an [`Url`], but
not always the other way around! This is because the format of the coordinates
and parameters after the URI scheme "geo:" may be invalid!

## License

geo-uri-rs is licensed under the MIT license (see the `LICENSE` file or
<http://opensource.org/licenses/MIT>).
