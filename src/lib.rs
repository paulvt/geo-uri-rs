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
pub enum CrsId {
    /// The WGS-84 coordinate reference system.
    Wgs84,
}

impl Default for CrsId {
    fn default() -> Self {
        Self::Wgs84
    }
}

/// Possible geo URI parse errors.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParseError {
    /// The geo URI is missing a proper scheme, i.e. the prefix `geo:`.
    #[error("Missing geo URI scheme")]
    MissingScheme,
    /// The geo URI contains no coordinates.
    #[error("Missing coordinates in geo URI")]
    MissingCoords,
    /// The geo URI lacks the latitude coordinate.
    #[error("Missing latitude coordinate in geo URI")]
    MissingLatitudeCoord,
    /// The geo URI lacks the longitude coordinate.
    #[error("Missing longitude coordinate in geo URI")]
    MissingLongitudeCoord,
    /// The geo URI contains an unparsable/invalid coordinate.
    #[error("Invalid coordinate in geo URI: {0}")]
    InvalidCoord(ParseFloatError),
    /// The geo URI contains an unsupported/invalid coordinate reference system.
    #[error("Invalid coordinate reference system")]
    InvalidCoordRefSystem,
    /// The geo URI contains an unparsable/invalid (uncertainty) distance.
    #[error("Invalid distance in geo URI: {0}")]
    InvalidDistance(ParseIntError),
}

/// A uniform resource identifier for geographic locations (geo URI).
///
/// TODO: Add examples!
///
/// # See also
///
/// For the proposed IEEE standard, see [RFC 5870](https://www.rfc-editor.org/rfc/rfc5870).
#[derive(Builder, Copy, Clone, Debug, Default)]
pub struct GeoUri {
    /// The coordinate reference system used by the coordinates of this URI.
    #[builder(default)]
    pub crs_id: CrsId,
    /// The latitude coordinate of a location.
    pub latitude: f64,
    /// The longitude coordinate of a location.
    pub longitude: f64,
    /// The altitude coordinate of a location, if provided.
    #[builder(default, setter(strip_option))]
    pub altitude: Option<f64>,
    #[builder(default, setter(strip_option))]
    /// The uncertainty around the location as a radius (distance) in meters.
    pub uncertainty: Option<u32>,
}

impl GeoUri {
    /// Return a builder to build a `GeoUri`.
    pub fn builder() -> GeoUriBuilder {
        GeoUriBuilder::default()
    }

    /// Try parsing a string into a `GeoUri`.
    ///
    /// For the geo URI scheme syntax, see the propsed IEEE standard
    /// [RFC 5870](https://www.rfc-editor.org/rfc/rfc5870#section-3.3).
    pub fn parse(uri: &str) -> Result<Self, ParseError> {
        let uri_path = uri.strip_prefix("geo:").ok_or(ParseError::MissingScheme)?;
        let mut parts = uri_path.split(';');

        // Parse the coordinate part.
        let coords_part = parts.next().expect("Split always yields at least one part");
        // Don't iterate over anything if the coordinate part is empty!
        let mut coords = if coords_part.is_empty() {
            return Err(ParseError::MissingCoords);
        } else {
            coords_part.splitn(3, ',')
        };
        let latitude = coords
            .next()
            .ok_or(ParseError::MissingLatitudeCoord) // This cannot really happen
            .and_then(|lat_s| lat_s.parse().map_err(ParseError::InvalidCoord))?;

        let longitude = coords
            .next()
            .ok_or(ParseError::MissingLongitudeCoord)
            .and_then(|lon_s| lon_s.parse().map_err(ParseError::InvalidCoord))?;

        let altitude = coords
            .next()
            .map(|alt_s| alt_s.parse().map_err(ParseError::InvalidCoord))
            .transpose()?;

        // Parse the remaining (parameters) parts.
        //
        // TODO: Handle possible casing issues in the pnames.
        // TODO: Handle percent encoding of the parameters.
        //
        // If the "crs" parameter is passed, its value must be "wgs84" or it is unsupported.
        // It can be followed by a "u" parameter or that can be the first one.
        // All other parameters are ignored.
        let mut param_parts = parts.flat_map(|part| part.split_once('='));
        let (crs_id, uncertainty) = match param_parts.next() {
            Some(("crs", value)) => {
                if value.to_ascii_lowercase() != "wgs84" {
                    return Err(ParseError::InvalidCoordRefSystem);
                }

                match param_parts.next() {
                    Some(("u", value)) => (
                        CrsId::Wgs84,
                        Some(value.parse().map_err(ParseError::InvalidDistance)?),
                    ),
                    Some(_) | None => (CrsId::Wgs84, None),
                }
            }
            Some(("u", value)) => (
                CrsId::default(),
                Some(value.parse().map_err(ParseError::InvalidDistance)?),
            ),
            Some(_) | None => (CrsId::default(), None),
        };

        Ok(GeoUri {
            crs_id,
            latitude,
            longitude,
            altitude,
            uncertainty,
        })
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
            write!(f, "{altitude}")?;
        }

        // Don't write the CRS since there is only one supported at the moment.
        if let Some(uncertainty) = self.uncertainty {
            write!(f, ";u={uncertainty}")?;
        }

        Ok(())
    }
}

impl FromStr for GeoUri {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<&str> for GeoUri {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl PartialEq for GeoUri {
    fn eq(&self, other: &Self) -> bool {
        // In the WGS-84 CRS the the longitude is ignored for the poles.
        let ignore_longitude = self.crs_id == CrsId::Wgs84 && self.latitude.abs() == 90.0;

        self.crs_id == other.crs_id
            && self.latitude == other.latitude
            && (ignore_longitude || self.longitude == other.longitude)
            && self.altitude == other.altitude
            && self.uncertainty == other.uncertainty
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn parse() -> Result<(), ParseError> {
        let geo_uri = GeoUri::parse("geo:52.107,5.134")?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_eq!(geo_uri.uncertainty, None);

        let geo_uri = GeoUri::parse("52.107,5.134");
        assert!(matches!(geo_uri, Err(ParseError::MissingScheme)));

        let geo_uri = GeoUri::parse("geo:geo:52.107,5.134");
        assert!(matches!(geo_uri, Err(ParseError::InvalidCoord(_))));

        let geo_uri = GeoUri::parse("geo:");
        assert!(matches!(geo_uri, Err(ParseError::MissingCoords)));

        let geo_uri = GeoUri::parse("geo:;u=5000");
        assert!(matches!(geo_uri, Err(ParseError::MissingCoords)));

        let geo_uri = GeoUri::parse("geo:52.107;u=1000");
        assert!(matches!(geo_uri, Err(ParseError::MissingLongitudeCoord)));

        let geo_uri = GeoUri::parse("geo:52.107,;u=1000");
        assert!(matches!(geo_uri, Err(ParseError::InvalidCoord(_))));

        let geo_uri = GeoUri::parse("geo:52.107,,6.00;u=1000");
        assert!(matches!(geo_uri, Err(ParseError::InvalidCoord(_))));

        let geo_uri = GeoUri::parse("geo:52.107,5.134,;u=1000");
        assert!(matches!(geo_uri, Err(ParseError::InvalidCoord(_))));

        let geo_uri = GeoUri::parse("geo:52.107,5.134,3.6")?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_float_eq!(geo_uri.altitude.unwrap(), 3.6, abs <= 0.001);
        assert_eq!(geo_uri.uncertainty, None);

        let geo_uri = GeoUri::parse("geo:52.107,5.34,6.00;u=");
        assert!(matches!(geo_uri, Err(ParseError::InvalidDistance(_))));

        let geo_uri = GeoUri::parse("geo:52.107,5.34,6.00;u=foo");
        assert!(matches!(geo_uri, Err(ParseError::InvalidDistance(_))));

        let geo_uri = GeoUri::parse("geo:52.107,5.34,6.00;crs=wgs84;u=foo");
        assert!(matches!(geo_uri, Err(ParseError::InvalidDistance(_))));

        let geo_uri = GeoUri::parse("geo:52.107,5.134,0.5;u=25000")?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_eq!(geo_uri.uncertainty, Some(25_000));

        let geo_uri = GeoUri::parse("geo:52.107,5.134,0.5;crs=wgs84;u=25000")?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_eq!(geo_uri.uncertainty, Some(25_000));

        let geo_uri = GeoUri::parse("geo:52.107,5.134,0.5;crs=wgs84;u=25000;foo=bar")?;
        assert_float_eq!(geo_uri.latitude, 52.107, abs <= 0.001);
        assert_float_eq!(geo_uri.longitude, 5.134, abs <= 0.001);
        assert_eq!(geo_uri.uncertainty, Some(25_000));

        let geo_uri = GeoUri::parse("geo:52.107,5.34,6.00;crs=foo");
        assert!(matches!(geo_uri, Err(ParseError::InvalidCoordRefSystem)));

        let geo_uri = GeoUri::parse("geo:52.107,5.34,6.00;crs=wgs84")?;
        assert!(matches!(geo_uri.crs_id, CrsId::Wgs84));

        // TODO: Add exmaples from RFC 5870!

        Ok(())
    }

    #[ignore]
    #[test]
    fn display() {
        todo!("Implement test");
    }

    #[ignore]
    #[test]
    fn from_str() {
        todo!("Implement test");
    }

    #[ignore]
    #[test]
    fn try_from() {
        todo!("Implement test");
    }

    #[ignore]
    #[test]
    fn partial_eq() {
        todo!("Implement test");
    }
}
