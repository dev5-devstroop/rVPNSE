use rvpnse::{
    client::{VpnClient, ClusterManager},
    config::{Config, LoadBalancingStrategy},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration with clustering enabled
    let mut config = Config::default_test();
    config.clustering.enabled = true;
    config.clustering.cluster_nodes = vec![
        "vpn-node1.example.com:443".to_string(),
        "vpn-node2.example.com:443".to_string(),
        "vpn-node3.example.com:443".to_string(),
    ];
    config.clustering.load_balancing_strategy = LoadBalancingStrategy::RoundRobin;
    config.clustering.max_peers_per_cluster = 100;
    config.clustering.connections_per_node = 10;

    println!("🔧 Testing SSL-VPN Clustering Support");
    println!("=====================================");

    // Test cluster manager creation
    let mut cluster_manager = ClusterManager::new(config.clustering.clone());
    println!("✅ Cluster manager created with {} nodes", cluster_manager.get_nodes_count());

    // Test peer count management
    println!("\n📊 Testing peer count management:");
    println!("   Initial peer count: {}", cluster_manager.get_peer_count());
    println!("   Can add peer: {}", cluster_manager.can_add_peer());

    cluster_manager.update_peer_count(25);
    println!("   Updated peer count to 25");
    println!("   Current peer count: {}", cluster_manager.get_peer_count());
    println!("   Can add peer: {}", cluster_manager.can_add_peer());

    cluster_manager.update_peer_count(99);
    println!("   Updated peer count to 99");
    println!("   Current peer count: {}", cluster_manager.get_peer_count());
    println!("   Can add peer: {}", cluster_manager.can_add_peer());

    cluster_manager.update_peer_count(100);
    println!("   Updated peer count to 100 (max)");
    println!("   Current peer count: {}", cluster_manager.get_peer_count());
    println!("   Can add peer: {}", cluster_manager.can_add_peer());

    // Test load balancing strategies
    println!("\n⚖️ Testing load balancing strategies:");
    for _ in 0..6 {
        if let Some(node) = cluster_manager.get_next_node() {
            println!("   Next node: {}", node.address);
        }
    }

    // Test VPN client with clustering
    println!("\n🔌 Testing VPN client with clustering:");
    let clustering_config = config.clustering.clone();
    let mut client = VpnClient::new(config)?;
    
    println!("   Initial peer count: {}", client.get_peer_count());
    client.update_peer_count(42);
    println!("   Updated peer count: {}", client.get_peer_count());
    
    if let Some(cluster_status) = client.get_cluster_status() {
        println!("   Cluster status:");
        for (address, healthy, connections) in cluster_status {
            println!("     - {}: healthy={}, connections={}", address, healthy, connections);
        }
    }

    // Test health check
    println!("\n🏥 Testing cluster health check:");
    match client.cluster_health_check().await {
        Ok(_) => println!("   ✅ Health check completed successfully"),
        Err(e) => println!("   ⚠️ Health check failed: {}", e),
    }

    println!("\n✅ Clustering test completed successfully!");
    println!("\n📋 Clustering Configuration Summary:");
    println!("   • Enabled: {}", clustering_config.enabled);
    println!("   • Nodes: {:?}", clustering_config.cluster_nodes);
    println!("   • Load Balancing: {:?}", clustering_config.load_balancing_strategy);
    println!("   • Max Peers: {}", clustering_config.max_peers_per_cluster);
    println!("   • Connections per Node: {}", clustering_config.connections_per_node);
    println!("   • Health Check Interval: {}s", clustering_config.health_check_interval);
    println!("   • Failover Timeout: {}s", clustering_config.failover_timeout);
    println!("   • RPC Version: {}", clustering_config.rpc_protocol_version);
    println!("   • Session Distribution: {:?}", clustering_config.session_distribution_mode);

    Ok(())
}
