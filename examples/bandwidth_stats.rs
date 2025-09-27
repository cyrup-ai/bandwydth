//! Example demonstrating NetBandwidthStats usage and bandwidth analysis
//!
//! This example shows how to use the bandwidth statistics methods for
//! performance monitoring and alerting.

use lib_bandwydth::{
    network::{NetBandwidthClass, NetBandwidthStats, NetOverallBandwidthStatus},
    start_monitor, MonitorConfig,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Bandwidth Statistics Analysis Example ===\n");

    // Shared state for statistics
    let stats_history = Arc::new(Mutex::new(Vec::new()));
    let stats_clone = Arc::clone(&stats_history);

    // Configure monitor
    let config = MonitorConfig {
        interface: None, // Monitor all interfaces
        resolve_dns: false,
        dns_server: None,
        allow_system_fallback: true,
        period_secs: 1,
    };

    // Start monitoring with analysis
    let handle = start_monitor(config, move |stats: NetBandwidthStats<10>| {
        // Demonstrate all NetBandwidthStats methods
        println!("--- Bandwidth Analysis ---");

        // 1. Get overall status
        let overall = stats.overall_status();
        println!("Overall Status: {overall:?}");

        // 2. Check if high performance
        if stats.is_high_performance() {
            println!("🚀 HIGH PERFORMANCE MODE DETECTED!");
        }

        // 3. Get human-readable description
        println!("Description: {}", stats.description());

        // 4. Get average speed
        let avg = stats.average_speed();
        println!("Average Speed: {avg:.2} Mbps");

        // 5. Check thresholds
        const MIN_ACCEPTABLE_MBPS: f64 = 50.0;
        const EXCELLENT_MBPS: f64 = 500.0;

        if stats.is_below_threshold(MIN_ACCEPTABLE_MBPS) {
            println!("⚠️  WARNING: Bandwidth below minimum threshold!");
        }

        if stats.is_above_threshold(EXCELLENT_MBPS) {
            println!("✨ EXCELLENT: Bandwidth exceeds excellence threshold!");
        }

        // Additional analysis
        println!("Current Speed: {:.2} Mbps", stats.current_speed);
        println!("Bandwidth Class: {:?}", stats.bandwidth_class);
        println!("Usage Status: {:?}", stats.usage_status);

        // Performance recommendations based on overall status
        match overall {
            NetOverallBandwidthStatus::Blazing => {
                println!("💫 Recommendation: Perfect for 8K streaming, large file transfers");
            }
            NetOverallBandwidthStatus::Good => {
                println!("✅ Recommendation: Great for 4K streaming, video conferencing");
            }
            NetOverallBandwidthStatus::Average => {
                println!("👍 Recommendation: Suitable for HD streaming, general usage");
            }
            NetOverallBandwidthStatus::Poor => {
                println!("⚡ Recommendation: May experience buffering, consider upgrade");
            }
            NetOverallBandwidthStatus::Inconclusive => {
                println!("❓ Recommendation: Gathering more data for analysis...");
            }
        }

        // Store for historical analysis
        if let Ok(mut history) = stats_clone.lock() {
            history.push(stats);

            // Keep only last 60 samples
            if history.len() > 60 {
                history.remove(0);
            }

            // Analyze trends
            if history.len() >= 10 {
                let recent_avg: f64 = history
                    .iter()
                    .rev()
                    .take(10)
                    .map(|s| s.current_speed)
                    .sum::<f64>()
                    / 10.0;

                let older_avg: f64 = history
                    .iter()
                    .rev()
                    .skip(10)
                    .take(10)
                    .map(|s| s.current_speed)
                    .sum::<f64>()
                    / 10.0_f64.max(1.0);

                if older_avg > 0.0 {
                    let change_pct = ((recent_avg - older_avg) / older_avg) * 100.0;
                    if change_pct > 20.0 {
                        println!("📈 Trend: Improving (+{change_pct:.1}%)");
                    } else if change_pct < -20.0 {
                        println!("📉 Trend: Degrading ({change_pct:.1}%)");
                    } else {
                        println!("➡️  Trend: Stable ({change_pct:+.1}%)");
                    }
                }
            }
        }

        println!();
    })?;

    println!("Monitoring bandwidth statistics... Press Ctrl+C to stop.\n");

    // Run for 30 seconds
    std::thread::sleep(Duration::from_secs(30));

    // Stop monitoring
    handle.stop();

    // Final analysis
    if let Ok(history) = stats_history.lock() {
        if !history.is_empty() {
            println!("\n=== Session Summary ===");

            let total_samples = history.len();
            let avg_speed: f64 =
                history.iter().map(|s| s.current_speed).sum::<f64>() / total_samples as f64;
            let max_speed = history
                .iter()
                .map(|s| s.current_speed)
                .fold(0.0_f64, |acc: f64, x| acc.max(x));
            let min_speed = history
                .iter()
                .map(|s| s.current_speed)
                .fold(f64::INFINITY, f64::min);

            println!("Total Samples: {total_samples}");
            println!("Average Speed: {avg_speed:.2} Mbps");
            println!("Peak Speed: {max_speed:.2} Mbps");
            println!("Minimum Speed: {min_speed:.2} Mbps");

            // Count performance levels
            let blazing_count = history
                .iter()
                .filter(|s| matches!(s.bandwidth_class, NetBandwidthClass::Blazing))
                .count();
            let good_count = history
                .iter()
                .filter(|s| matches!(s.bandwidth_class, NetBandwidthClass::Good))
                .count();
            let average_count = history
                .iter()
                .filter(|s| matches!(s.bandwidth_class, NetBandwidthClass::Average))
                .count();
            let poor_count = history
                .iter()
                .filter(|s| matches!(s.bandwidth_class, NetBandwidthClass::Poor))
                .count();

            println!("\nPerformance Distribution:");
            if blazing_count > 0 {
                println!(
                    "  Blazing: {} samples ({:.1}%)",
                    blazing_count,
                    (blazing_count as f64 / total_samples as f64) * 100.0
                );
            }
            if good_count > 0 {
                println!(
                    "  Good: {} samples ({:.1}%)",
                    good_count,
                    (good_count as f64 / total_samples as f64) * 100.0
                );
            }
            if average_count > 0 {
                println!(
                    "  Average: {} samples ({:.1}%)",
                    average_count,
                    (average_count as f64 / total_samples as f64) * 100.0
                );
            }
            if poor_count > 0 {
                println!(
                    "  Poor: {} samples ({:.1}%)",
                    poor_count,
                    (poor_count as f64 / total_samples as f64) * 100.0
                );
            }

            // High performance time
            let high_perf_count = history.iter().filter(|s| s.is_high_performance()).count();
            println!(
                "\nHigh Performance Time: {:.1}%",
                (high_perf_count as f64 / total_samples as f64) * 100.0
            );
        }
    }

    println!("\nExample completed!");
    Ok(())
}
