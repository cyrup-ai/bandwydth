use std::fmt::Write;

use insta::assert_snapshot;
use itertools::Itertools;
use lib_bandwydth::{
    cli::UnitFamily,
    display::{BandwidthUnitFamily, DisplayBandwidth},
};

#[test]
fn bandwidth_formatting() {
    let test_bandwidths_formatted = vec![
        UnitFamily::BinBytes,
        UnitFamily::BinBits,
        UnitFamily::SiBytes,
        UnitFamily::SiBits,
    ]
    .into_iter()
    .map(BandwidthUnitFamily::from)
    .cartesian_product(
        // I feel like this is a decent selection of values
        (-6..60)
            .map(|exp| 2f64.powi(exp))
            .chain((-5..45).map(|exp| 2.5f64.powi(exp)))
            .chain((-4..38).map(|exp| 3f64.powi(exp)))
            .chain((-3..26).map(|exp| 5f64.powi(exp))),
    )
    .map(|(unit_family, bandwidth)| DisplayBandwidth {
        bandwidth,
        unit_family,
    })
    .fold(String::new(), |mut buf, b| {
        let _ = writeln!(buf, "{b:?}: {b}");
        buf
    });

    assert_snapshot!(test_bandwidths_formatted);
}
