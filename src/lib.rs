#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    trivial_casts,
    trivial_numeric_casts,
    renamed_and_removed_lints,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![deny(missing_docs)]

use std::fmt;
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;

use derive_builder::Builder;
use thiserror::Error;

/// The scheme name of a geo URI.
const URI_SCHEME_NAME: &str = "geo";

/// Possible geo URI errors.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum Error {
    /// The geo URI contains an unparsable/invalid coordinate.
    #[error("Invalid coordinate in geo URI: {0}")]
    InvalidCoord(ParseFloatError),

    /// The geo URI contains an unsupported/invalid coordinate reference system.
    #[error("Invalid coordinate reference system")]
    InvalidCoordRefSystem,

    /// The geo URI contains an unparsable/invalid (uncertainty) distance.
    #[error("Invalid distance in geo URI: {0}")]
    InvalidDistance(ParseIntError),

    /// The geo URI contains no coordinates.
    #[error("Missing coordinates in geo URI")]
    MissingCoords,

    /// The geo URI lacks the latitude coordinate.
    #[error("Missing latitude coordinate in geo URI")]
    MissingLatitude,

    /// The geo URI lacks the longitude coordinate.
    #[error("Missing longitude coordinate in geo URI")]
    MissingLongitude,

    /// The geo URI is missing a proper scheme, i.e. the prefix `geo:`.
    #[error("Missing geo URI scheme")]
    MissingScheme,

    /// The latitude coordinate is out of range of `-90.0..=90.0` degrees.
    ///
    /// This can only fail for the WGS-84 coordinate reference system.
    #[error("Latitude coordinate is out of range")]
    OutOfRangeLatitude,

    /// The longitude coordinate is out of range of `-180.0..=180.0` degrees
    ///
    /// This can only fail for the WGS-84 coordinate reference system.
    #[error("Longitude coordinate is out of range")]
    OutOfRangeLongitude,
}

/// The reference system of the provided coordinates.
///
/// Currently only the `WGS-84` coordinate reference system is supported.
/// It defines the latitude and longitude of the [`GeoUri`] to be in decimal degrees and the
/// altitude in meters.
///
/// For more details see the
/// [component description](ttps://www.rfc-editor.org/rfc/rfc5870#section-3.4.2) in
/// [RFC 5870](https://www.rfc-editor.org/rfc/rfc5870).
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CoordRefSystem {
    /// The WGS-84 coordinate reference system.
    Wgs84,
}

impl CoordRefSystem {
    /// Validates geo location coordinates against the selected coordinate reference system.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geo_uri::{CoordRefSystem, Error};
    /// let crs = CoordRefSystem::Wgs84;
    /// assert_eq!(crs.validate(52.107, 5.134), Ok(()));
    /// assert_eq!(
    ///     crs.validate(100.0, 5.134), // Latitude not in range `-90.0..=90.0`!
    ///     Err(Error::OutOfRangeLatitude)
    /// );
    /// assert_eq!(
    ///     crs.validate(51.107, -200.0), // Longitude not in range `-180.0..=180.0`!
    ///     Err(Error::OutOfRangeLongitude)
    /// );
    /// ```
    pub fn validate(&self, latitude: f64, longitude: f64) -> Result<(), Error> {
        // This holds only for WGS-84, but it is the only one supported right now!
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(Error::OutOfRangeLatitude);
        }

        // This holds only for WGS-84, but it is the only one supported right now!
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(Error::OutOfRangeLongitude);
        }

        Ok(())
    }
}

impl Default for CoordRefSystem {
    fn default() -> Self {
        Self::Wgs84
    }
}

/// A uniform resource identifier for geographic locations (geo URI).
///
/// # Examples
///
/// ## Parsing
///
/// You can get a [`GeoUri`] by converting it from a geo URI string ([`&str`]):
///
/// ```rust
/// use geo_uri::GeoUri;
///
/// let geo_uri = GeoUri::try_from("geo:52.107,5.134,3.6;u=1000").expect("valid geo URI");
/// assert_eq!(geo_uri.latitude(), 52.107);
/// assert_eq!(geo_uri.longitude(), 5.134);
/// assert_eq!(geo_uri.altitude(), Some(3.6));
/// assert_eq!(geo_uri.uncertainty(), Some(1000));
/// ```
///
/// or by using the [`TryFrom`] trait:
/// ```
/// use geo_uri::GeoUri;
/// use std::str::FromStr;
///
/// let geo_uri = GeoUri::from_str("geo:52.107,5.134;u=2000").expect("valid geo URI");
/// assert_eq!(geo_uri.latitude(), 52.107);
/// assert_eq!(geo_uri.longitude(), 5.134);
/// assert_eq!(geo_uri.altitude(), None);
/// assert_eq!(geo_uri.uncertainty(), Some(2000));
/// ```
///
/// It is also possible to call the parse function directly:
///
/// ```rust
/// use geo_uri::GeoUri;
///
/// let geo_uri = GeoUri::parse("geo:52.107,5.134,3.6").expect("valid geo URI");
/// assert_eq!(geo_uri.latitude(), 52.107);
/// assert_eq!(geo_uri.longitude(), 5.134);
/// assert_eq!(geo_uri.altitude(), Some(3.6));
/// assert_eq!(geo_uri.uncertainty(), None);
/// ```
///
/// ## Generating
///
/// To get an geo URI string from some coordinates, use the [`GeoUriBuilder`]:
///
/// ```rust
/// use geo_uri::GeoUri;
///
/// let geo_uri = GeoUri::builder()
///     .latitude(52.107)
///     .longitude(5.134)
///     .uncertainty(1_000)
///     .build()
///     .unwrap();
/// assert_eq!(
///     geo_uri.to_string(),
///     String::from("geo:52.107,5.134;u=1000")
/// );
/// assert_eq!(
///     format!("{geo_uri}"),
///     String::from("geo:52.107,5.134;u=1000")
/// );
/// ```
///
/// # See also
///
/// For the proposed IEEE standard, see [RFC 5870](https://www.rfc-editor.org/rfc/rfc5870).
#[derive(Builder, Copy, Clone, Debug, Default)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct GeoUri {
    /// The coordinate reference system used by the coordinates of this URI.
    #[builder(default)]
    crs: CoordRefSystem,

    /// The latitude coordinate of a location.
    ///
    /// For the WGS-84 coordinate reference system, this should be in the range of
    /// `-90.0` up until including `90.0` degrees.
    latitude: f64,

    /// The longitude coordinate of a location.
    ///
    /// For the WGS-84 coordinate reference system, this should be in the range of
    /// `-180.0` up until including `180.0` degrees.
    longitude: f64,

    /// The altitude coordinate of a location, if provided.
    #[builder(default, setter(strip_option))]
    altitude: Option<f64>,

    #[builder(default, setter(strip_option))]
    /// The uncertainty around the location as a radius (distance) in meters.
    uncertainty: Option<u32>,
}

impl GeoUri {
    /// Return a builder for `GeoUri`.
    pub fn builder() -> GeoUriBuilder {
        GeoUriBuilder::default()
    }

    /// Try parsing a geo URI string into a `GeoUri`.
    ///
    /// For the geo URI scheme syntax, see the propsed IEEE standard
    /// [RFC 5870](https://www.rfc-editor.org/rfc/rfc5870#section-3.3).
    pub fn parse(uri: &str) -> Result<Self, Error> {
        let uri = uri.to_ascii_lowercase();
        let uri_path = uri.strip_prefix("geo:").ok_or(Error::MissingScheme)?;
        let mut parts = uri_path.split(';');

        // Parse the coordinate part.
        let coords_part = parts.next().expect("Split always yields at least one part");
        // Don't iterate over anything if the coordinate part is empty!
        let mut coords = if coords_part.is_empty() {
            return Err(Error::MissingCoords);
        } else {
            coords_part.splitn(3, ',')
        };
        let latitude = coords
            .next()
            .ok_or(Error::MissingLatitude) // This cannot really happen
            .and_then(|lat_s| lat_s.parse().map_err(Error::InvalidCoord))?;

        let longitude = coords
            .next()
            .ok_or(Error::MissingLongitude)
            .and_then(|lon_s| lon_s.parse().map_err(Error::InvalidCoord))?;

        let altitude = coords
            .next()
            .map(|alt_s| alt_s.parse().map_err(Error::InvalidCoord))
            .transpose()?;

        // Parse the remaining (parameters) parts.
        //
        // TODO: Handle percent encoding of the parameters.
        //
        // If the "crs" parameter is passed, its value must be "wgs84" or it is unsupported.
        // It can be followed by a "u" parameter or that can be the first one.
        // All other parameters are ignored.
        let mut param_parts = parts.flat_map(|part| part.split_once('='));
        let (crs, uncertainty) = match param_parts.next() {
            Some(("crs", value)) => {
                if value != "wgs84" {
                    return Err(Error::InvalidCoordRefSystem);
                }

                match param_parts.next() {
                    Some(("u", value)) => (
                        CoordRefSystem::Wgs84,
                        Some(value.parse().map_err(Error::InvalidDistance)?),
                    ),
                    Some(_) | None => (CoordRefSystem::Wgs84, None),
                }
            }
            Some(("u", value)) => (
                CoordRefSystem::default(),
                Some(value.parse().map_err(Error::InvalidDistance)?),
            ),
            Some(_) | None => (CoordRefSystem::default(), None),
        };

        crs.validate(latitude, longitude)?;

        Ok(GeoUri {
            crs,
            latitude,
            longitude,
            altitude,
            uncertainty,
        })
    }

    /// Returns the latitude coordinate.
    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    /// Changes the latitude coordinate.
    ///
    /// The latitude may be out of range for the coordinate reference system.
    pub fn set_latitude(&mut self, latitude: f64) -> Result<(), Error> {
        self.crs.validate(latitude, self.longitude)?;
        self.latitude = latitude;

        Ok(())
    }

    /// Returns the longitude coordinate.
    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    /// Changes the longitude coordinate.
    ///
    /// The longitude may be out of range for the coordinate reference system.
    pub fn set_longitude(&mut self, longitude: f64) -> Result<(), Error> {
        self.crs.validate(self.latitude, longitude)?;
        self.longitude = longitude;

        Ok(())
    }

    /// Returns the altitude coordinate (if any).
    pub fn altitude(&self) -> Option<f64> {
        self.altitude
    }

    /// Changes the altitude coordinate.
    pub fn set_altitude(&mut self, altitude: Option<f64>) {
        self.altitude = altitude;
    }

    /// Returns the uncertainty around the location.
    pub fn uncertainty(&self) -> Option<u32> {
        self.uncertainty
    }

    /// Changes the uncertainty around the location.
    pub fn set_uncertainty(&mut self, uncertainty: Option<u32>) {
        self.uncertainty = uncertainty;
    }
}

impl fmt::Display for GeoUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            latitude,
            longitude,
            ..
        } = self;
        write!(f, "{URI_SCHEME_NAME}:{latitude},{longitude}")?;

        if let Some(altitude) = self.altitude {
            write!(f, ",{altitude}")?;
        }

        // Don't write the CRS since there is only one supported at the moment.
        if let Some(uncertainty) = self.uncertainty {
            write!(f, ";u={uncertainty}")?;
        }

        Ok(())
    }
}

impl FromStr for GeoUri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<&str> for GeoUri {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl PartialEq for GeoUri {
    fn eq(&self, other: &Self) -> bool {
        // In the WGS-84 CRS the the longitude is ignored for the poles.
        let ignore_longitude = self.crs == CoordRefSystem::Wgs84 && self.latitude.abs() == 90.0;

        self.crs == other.crs
            && self.latitude == other.latitude
            && (ignore_longitude || self.longitude == other.longitude)
            && self.altitude == other.altitude
            && self.uncertainty == other.uncertainty
    }
}

impl GeoUriBuilder {
    /// Validates the coordinates against the
    fn validate(&self) -> Result<(), String> {
        self.crs
            .unwrap_or_default()
            .validate(
                self.latitude.unwrap_or_default(),
                self.longitude.unwrap_or_default(),
            )
            .map_err(|e| format!("{e}"))
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn coord_ref_system_default() {
        assert_eq!(CoordRefSystem::default(), CoordRefSystem::Wgs84);
    }

    #[test]
    fn coord_ref_system_validate() {
        let crs = CoordRefSystem::Wgs84;
        assert_eq!(crs.validate(52.107, 5.134), Ok(()));
        assert_eq!(crs.validate(100.0, 5.134), Err(Error::OutOfRangeLatitude));
        assert_eq!(
            crs.validate(51.107, -200.0),
            Err(Error::OutOfRangeLongitude)
        );
    }

    #[test]
    fn geo_uri_builder() -> Result<(), GeoUriBuilderError> {
        let mut builder = GeoUri::builder();
        assert!(matches!(
            builder.build(),
            Err(GeoUriBuilderError::UninitializedField("latitude"))
        ));

        builder.latitude(52.107);
        assert!(matches!(
            builder.build(),
            Err(GeoUriBuilderError::UninitializedField("longitude"))
        ));

        builder.longitude(5.134);
        let geo_uri = builder.build()?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_eq!(geo_uri.altitude, None);
        assert_eq!(geo_uri.uncertainty, None);

        builder.latitude(100.0);
        assert!(matches!(
            builder.build(),
            Err(GeoUriBuilderError::ValidationError(_))
        ));

        builder.latitude(52.107).longitude(-200.0);
        assert!(matches!(
            builder.build(),
            Err(GeoUriBuilderError::ValidationError(_))
        ));

        Ok(())
    }

    #[test]
    fn geo_uri_parse() -> Result<(), Error> {
        let geo_uri = GeoUri::parse("geo:52.107,5.134")?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_eq!(geo_uri.altitude, None);
        assert_eq!(geo_uri.uncertainty, None);

        let geo_uri = GeoUri::parse("52.107,5.134");
        assert!(matches!(geo_uri, Err(Error::MissingScheme)));

        let geo_uri = GeoUri::parse("geo:geo:52.107,5.134");
        assert!(matches!(geo_uri, Err(Error::InvalidCoord(_))));

        let geo_uri = GeoUri::parse("geo:");
        assert!(matches!(geo_uri, Err(Error::MissingCoords)));

        let geo_uri = GeoUri::parse("geo:;u=5000");
        assert!(matches!(geo_uri, Err(Error::MissingCoords)));

        let geo_uri = GeoUri::parse("geo:52.107;u=1000");
        assert!(matches!(geo_uri, Err(Error::MissingLongitude)));

        let geo_uri = GeoUri::parse("geo:52.107,;u=1000");
        assert!(matches!(geo_uri, Err(Error::InvalidCoord(_))));

        let geo_uri = GeoUri::parse("geo:52.107,,6.50;u=1000");
        assert!(matches!(geo_uri, Err(Error::InvalidCoord(_))));

        let geo_uri = GeoUri::parse("geo:52.107,5.134,;u=1000");
        assert!(matches!(geo_uri, Err(Error::InvalidCoord(_))));

        let geo_uri = GeoUri::parse("geo:52.107,5.134,3.6")?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_float_eq!(geo_uri.altitude.unwrap(), 3.6, abs <= 0.001);
        assert_eq!(geo_uri.uncertainty, None);

        let geo_uri = GeoUri::parse("geo:52.107,5.34,3.6;u=");
        assert!(matches!(geo_uri, Err(Error::InvalidDistance(_))));

        let geo_uri = GeoUri::parse("geo:52.107,5.34,3.6;u=foo");
        assert!(matches!(geo_uri, Err(Error::InvalidDistance(_))));

        let geo_uri = GeoUri::parse("geo:52.107,5.34,3.6;crs=wgs84;u=foo");
        assert!(matches!(geo_uri, Err(Error::InvalidDistance(_))));

        let geo_uri = GeoUri::parse("geo:52.107,5.134,3.6;u=25000")?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_float_eq!(geo_uri.altitude.unwrap(), 3.6, abs <= 0.001);
        assert_eq!(geo_uri.uncertainty, Some(25_000));

        let geo_uri = GeoUri::parse("geo:52.107,5.134,3.6;crs=wgs84;u=25000")?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_float_eq!(geo_uri.altitude.unwrap(), 3.6, abs <= 0.001);
        assert_eq!(geo_uri.uncertainty, Some(25_000));

        let geo_uri = GeoUri::parse("geo:52.107,5.134,3.6;CRS=wgs84;U=25000")?;
        assert_eq!(geo_uri.uncertainty, Some(25_000));

        let geo_uri = GeoUri::parse("geo:52.107,5.134,3.6;crs=wgs84;u=25000;foo=bar")?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_float_eq!(geo_uri.altitude.unwrap(), 3.6, abs <= 0.001);
        assert_eq!(geo_uri.uncertainty, Some(25_000));

        let geo_uri = GeoUri::parse("geo:52.107,5.34,3.6;crs=foo");
        assert!(matches!(geo_uri, Err(Error::InvalidCoordRefSystem)));

        let geo_uri = GeoUri::parse("geo:52.107,5.34,3.6;crs=wgs84")?;
        assert!(matches!(geo_uri.crs, CoordRefSystem::Wgs84));

        // TODO: Add exmaples from RFC 5870!

        Ok(())
    }

    #[test]
    fn geo_uri_get_set() {
        let mut geo_uri = GeoUri {
            crs: CoordRefSystem::Wgs84,
            latitude: 52.107,
            longitude: 5.134,
            altitude: None,
            uncertainty: None,
        };
        assert_eq!(geo_uri.latitude(), 52.107);
        assert_eq!(geo_uri.longitude(), 5.134);
        assert_eq!(geo_uri.altitude(), None);
        assert_eq!(geo_uri.uncertainty(), None);

        assert_eq!(geo_uri.set_latitude(53.107), Ok(()));
        assert_eq!(geo_uri.set_latitude(100.0), Err(Error::OutOfRangeLatitude));
        assert_eq!(geo_uri.latitude(), 53.107);

        assert_eq!(geo_uri.set_longitude(6.134), Ok(()));
        assert_eq!(
            geo_uri.set_longitude(-200.0),
            Err(Error::OutOfRangeLongitude)
        );
        assert_eq!(geo_uri.longitude(), 6.134);

        geo_uri.set_altitude(Some(3.6));
        assert_eq!(geo_uri.altitude(), Some(3.6));

        geo_uri.set_uncertainty(Some(25_000));
        assert_eq!(geo_uri.uncertainty(), Some(25_000));
    }

    #[test]
    fn geo_uri_display() {
        let mut geo_uri = GeoUri {
            crs: CoordRefSystem::Wgs84,
            latitude: 52.107,
            longitude: 5.134,
            altitude: None,
            uncertainty: None,
        };
        assert_eq!(&geo_uri.to_string(), "geo:52.107,5.134");

        geo_uri.altitude = Some(3.6);
        assert_eq!(&geo_uri.to_string(), "geo:52.107,5.134,3.6");

        geo_uri.uncertainty = Some(25_000);
        assert_eq!(&geo_uri.to_string(), "geo:52.107,5.134,3.6;u=25000");
    }

    #[test]
    fn geo_uri_from_str() -> Result<(), Error> {
        let geo_uri = GeoUri::from_str("geo:52.107,5.134")?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_eq!(geo_uri.altitude, None);
        assert_eq!(geo_uri.uncertainty, None);

        Ok(())
    }

    #[test]
    fn geo_uri_try_from() -> Result<(), Error> {
        let geo_uri = GeoUri::try_from("geo:52.107,5.134")?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_eq!(geo_uri.altitude, None);
        assert_eq!(geo_uri.uncertainty, None);

        Ok(())
    }

    #[test]
    fn geo_uri_partial_eq() -> Result<(), GeoUriBuilderError> {
        let geo_uri = GeoUri::builder()
            .latitude(52.107)
            .longitude(5.134)
            .build()?;
        let geo_uri2 = GeoUri::builder()
            .latitude(52.107)
            .longitude(5.134)
            .build()?;
        assert_eq!(geo_uri, geo_uri2);
        assert_eq!(geo_uri, geo_uri.clone());

        let geo_uri = GeoUri::builder().latitude(90.0).longitude(5.134).build()?;
        let geo_uri2 = GeoUri::builder().latitude(90.0).longitude(5.134).build()?;
        assert_eq!(geo_uri, geo_uri2);

        let geo_uri = GeoUri::builder().latitude(-90.0).longitude(5.134).build()?;
        let geo_uri2 = GeoUri::builder().latitude(-90.0).longitude(5.134).build()?;
        assert_eq!(geo_uri, geo_uri2);

        Ok(())
    }
}
