//! Positions sources API

use crate::DevicePosition;
use time::OffsetDateTime;

/// Position source
pub trait PositionsSource {
    /// Fetch the raw positing during the period
    fn fetch(
        &mut self,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<DevicePosition>, String>;
}

