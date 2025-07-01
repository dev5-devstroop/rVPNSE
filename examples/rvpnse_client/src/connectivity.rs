//! Network connectivity testing utilities

use rvpnse::Result;
use std::time::Duration;

/// Network connectivity tester
pub struct ConnectivityTester {
    client: reqwest::Client,
}

impl ConnectivityTester {
    /// Create a new connectivity tester
    pub fn new(timeout: Duration) -> rvpnse::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| rvpnse::VpnError::Network(format!("HTTP client error: {}", e)))?;

        Ok(Self { client })
    }

    /// Get current public IP address using multiple methods
    pub async fn get_public_ip(&self) -> Result<String> {
        // Try the public-ip crate first (fast and reliable)
        if let Some(ip) = public_ip::addr().await {
            return Ok(ip.to_string());
        }

        // Fallback to HTTP-based IP detection services
        let services = [
            "https://api.ipify.org",
            "https://icanhazip.com", 
            "https://ipecho.net/plain",
            "https://checkip.amazonaws.com",
        ];

        for service in &services {
            if let Ok(response) = self.client.get(*service).send().await {
                if let Ok(ip_text) = response.text().await {
                    let ip = ip_text.trim().to_string();
                    if !ip.is_empty() && Self::is_valid_ip(&ip) {
                        return Ok(ip);
                    }
                }
            }
        }

        Err(rvpnse::VpnError::Network("Failed to detect public IP address".to_string()))
    }

    /// Test connectivity to major websites
    pub async fn test_connectivity(&self) -> Result<ConnectivityReport> {
        let test_sites = [
            ("Google", "https://www.google.com"),
            ("Cloudflare", "https://www.cloudflare.com"),
            ("HTTPBin", "https://httpbin.org/get"),
        ];

        let mut report = ConnectivityReport::new();

        for (name, url) in &test_sites {
            let start = std::time::Instant::now();
            
            match self.client.head(*url).send().await {
                Ok(response) => {
                    let duration = start.elapsed();
                    let status = response.status();
                    
                    if status.is_success() {
                        report.add_success(name, url, duration);
                        println!("   ✅ {} ({:?})", name, duration);
                    } else {
                        report.add_failure(name, url, format!("HTTP {}", status));
                        println!("   ❌ {} (HTTP {})", name, status);
                    }
                },
                Err(e) => {
                    report.add_failure(name, url, e.to_string());
                    println!("   ❌ {} ({})", name, e);
                }
            }
        }

        if report.success_count > 0 {
            println!("   📊 Connectivity: {}/{} sites reachable", 
                     report.success_count, report.total_tests);
            Ok(report)
        } else {
            Err(rvpnse::VpnError::Network("No connectivity to test sites".to_string()))
        }
    }

    /// Validate if a string is a valid IP address
    fn is_valid_ip(ip: &str) -> bool {
        use std::net::IpAddr;
        ip.parse::<IpAddr>().is_ok()
    }
}

/// Connectivity test report
#[derive(Debug)]
pub struct ConnectivityReport {
    pub success_count: u32,
    pub failure_count: u32,
    pub total_tests: u32,
    pub average_latency: Option<Duration>,
}

impl ConnectivityReport {
    fn new() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            total_tests: 0,
            average_latency: None,
        }
    }

    fn add_success(&mut self, _name: &str, _url: &str, duration: Duration) {
        self.success_count += 1;
        self.total_tests += 1;
        
        // Calculate running average latency
        if let Some(avg) = self.average_latency {
            self.average_latency = Some((avg + duration) / 2);
        } else {
            self.average_latency = Some(duration);
        }
    }

    fn add_failure(&mut self, _name: &str, _url: &str, _error: String) {
        self.failure_count += 1;
        self.total_tests += 1;
    }
}
