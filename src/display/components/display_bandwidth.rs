use std::fmt;

use derive_more::Debug;

use crate::cli::UnitFamily;

/// Bandwidth value with associated unit family for display formatting.
#[derive(Copy, Clone, Debug)]
pub struct DisplayBandwidth {
    /// The bandwidth value in bytes.
    // Custom format for reduced precision.
    // Workaround for FP calculation discrepancy between Unix and Windows.
    // See https://github.com/rust-lang/rust/issues/111405#issuecomment-2055964223.
    #[debug("{bandwidth:.10e}")]
    pub bandwidth: f64,
    /// The unit family to use for display formatting.
    pub unit_family: BandwidthUnitFamily,
}

impl fmt::Display for DisplayBandwidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (div, suffix) = self.unit_family.get_unit_for(self.bandwidth);
        write!(f, "{:.2}{suffix}", self.bandwidth / div)
    }
}

/// Type wrapper around [`UnitFamily`] to provide extra functionality.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[debug("{_0:?}")]
pub struct BandwidthUnitFamily(pub UnitFamily);

impl From<UnitFamily> for BandwidthUnitFamily {
    fn from(unit_family: UnitFamily) -> Self {
        Self(unit_family)
    }
}

impl BandwidthUnitFamily {
    /// Create a new BandwidthUnitFamily from a UnitFamily.
    pub fn new(unit_family: UnitFamily) -> Self {
        Self(unit_family)
    }

    /// Get the inner UnitFamily.
    pub fn inner(&self) -> UnitFamily {
        self.0
    }

    #[inline]
    /// Returns an array of tuples, corresponding to the steps of this unit family.
    ///
    /// Each step contains a divisor, an upper bound, and a unit suffix.
    fn steps(&self) -> [(f64, f64, &'static str); 6] {
        /// The fraction of the next unit the value has to meet to step up.
        const STEP_UP_FRAC: f64 = 0.95;
        /// Binary base: 2^10.
        const BB: f64 = 1024.0;

        use UnitFamily as F;
        // probably could macro this stuff, but I'm too lazy
        match self.0 {
            F::BinBytes => [
                (1.0, BB * STEP_UP_FRAC, "B"),
                (BB, BB.powi(2) * STEP_UP_FRAC, "KiB"),
                (BB.powi(2), BB.powi(3) * STEP_UP_FRAC, "MiB"),
                (BB.powi(3), BB.powi(4) * STEP_UP_FRAC, "GiB"),
                (BB.powi(4), BB.powi(5) * STEP_UP_FRAC, "TiB"),
                (BB.powi(5), f64::MAX, "PiB"),
            ],
            F::BinBits => [
                (1.0 / 8.0, BB / 8.0 * STEP_UP_FRAC, "b"),
                (BB / 8.0, BB.powi(2) / 8.0 * STEP_UP_FRAC, "Kib"),
                (BB.powi(2) / 8.0, BB.powi(3) / 8.0 * STEP_UP_FRAC, "Mib"),
                (BB.powi(3) / 8.0, BB.powi(4) / 8.0 * STEP_UP_FRAC, "Gib"),
                (BB.powi(4) / 8.0, BB.powi(5) / 8.0 * STEP_UP_FRAC, "Tib"),
                (BB.powi(5) / 8.0, f64::MAX, "Pib"),
            ],
            F::SiBytes => [
                (1.0, 1e3 * STEP_UP_FRAC, "B"),
                (1e3, 1e6 * STEP_UP_FRAC, "kB"),
                (1e6, 1e9 * STEP_UP_FRAC, "MB"),
                (1e9, 1e12 * STEP_UP_FRAC, "GB"),
                (1e12, 1e15 * STEP_UP_FRAC, "TB"),
                (1e15, f64::MAX, "PB"),
            ],
            F::SiBits => [
                (1.0 / 8.0, 1e3 / 8.0 * STEP_UP_FRAC, "b"),
                (1e3 / 8.0, 1e6 / 8.0 * STEP_UP_FRAC, "kb"),
                (1e6 / 8.0, 1e9 / 8.0 * STEP_UP_FRAC, "Mb"),
                (1e9 / 8.0, 1e12 / 8.0 * STEP_UP_FRAC, "Gb"),
                (1e12 / 8.0, 1e15 / 8.0 * STEP_UP_FRAC, "Tb"),
                (1e15 / 8.0, f64::MAX, "Pb"),
            ],
        }
    }

    /// Select a unit for a given value, returning its divisor and suffix.
    fn get_unit_for(&self, bytes: f64) -> (f64, &'static str) {
        let Some((div, _, suffix)) = self
            .steps()
            .into_iter()
            .find(|&(_, bound, _)| bound >= bytes)
        else {
            log::error!("Cannot select an appropriate unit for {bytes:.2}B, using largest available unit");
            // Return the largest unit as fallback instead of panicking
            let steps = self.steps();
            let (div, _, suffix) = steps[steps.len() - 1];
            return (div, suffix);
        };

        (div, suffix)
    }
}
