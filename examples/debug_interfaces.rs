//! Debug interfaces example - diagnose network interface discovery and packet capture issues
//!
//! This example performs comprehensive diagnostics to identify why network monitoring might fail:
//! - Lists all discovered network interfaces  
//! - Tests interface filtering logic
//! - Attempts datalink channel creation for each interface
//! - Tests actual packet capture for 5 seconds
//! - Provides detailed failure analysis

use lib_bandwydth::debug_interfaces::{diagnose_network_interfaces, test_get_input};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see all debug output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    
    println!("🔍 Network Interface Diagnostic Tool");
    println!("====================================\n");
    
    // Test interface discovery and packet capture
    diagnose_network_interfaces(None);
    
    println!("\n📋 Testing get_input() function directly...");
    match test_get_input(None, false, None) {
        Ok(_) => println!("✅ get_input() test passed!"),
        Err(e) => {
            println!("❌ get_input() test failed: {}", e);
            println!("\n💡 Troubleshooting suggestions:");
            println!("1. Run with sudo for elevated privileges");
            println!("2. Check if any interfaces are up and have IP addresses"); 
            println!("3. Try specifying a specific interface with --interface");
            println!("4. On macOS, check if SIP is blocking raw socket access");
            println!("5. Verify network interfaces exist: ip link show (Linux) or ifconfig (macOS)");
        }
    }
    
    Ok(())
}