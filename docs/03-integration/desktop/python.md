# 🐍 Python Integration Guide

Complete guide for integrating rVPNSE into Python applications. This guide covers both the Python wrapper library and direct FFI usage for maximum flexibility.

## 📋 Prerequisites

- Python 3.8 or later
- pip package manager
- C compiler (for building native extensions)
- Platform-specific requirements:
  - **Windows**: Visual Studio Build Tools
  - **macOS**: Xcode Command Line Tools
  - **Linux**: GCC and development headers

## 🚀 Installation

### Option 1: Install from PyPI (Recommended)
```bash
pip install rvpnse
```

### Option 2: Install from Source
```bash
git clone https://github.com/devstroop/rvpnse.git
cd rvpnse/bindings/python
pip install -e .
```

### Option 3: Use Pre-built Wheels
```bash
# Download platform-specific wheel
pip install https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse-1.0.0-cp38-cp38-linux_x86_64.whl
```

## 💻 Basic Usage

### 1. Simple Connection

```python
import asyncio
from rvpnse import RvpnseClient, RvpnseConfig

async def simple_vpn_connection():
    # Create configuration
    config = RvpnseConfig(
        server_host="vpn.example.com",
        server_port=443,
        username="your_username",
        password="your_password",
        hub_name="VPN"
    )
    
    # Create client and connect
    client = RvpnseClient(config)
    
    try:
        await client.connect()
        print("✅ Connected to VPN!")
        
        # Keep connection alive
        await asyncio.sleep(60)
        
    except Exception as e:
        print(f"❌ Connection failed: {e}")
    finally:
        await client.disconnect()
        print("🔌 Disconnected from VPN")

# Run the example
asyncio.run(simple_vpn_connection())
```

### 2. Configuration from File

```python
from rvpnse import RvpnseConfig, RvpnseClient

# Load configuration from TOML file
config = RvpnseConfig.from_file("config/vpn_config.toml")

# Or load from dictionary
config_dict = {
    "server_host": "vpn.example.com",
    "server_port": 443,
    "username": "user",
    "password": "pass",
    "hub_name": "VPN",
    "enable_compression": True,
    "keep_alive_interval": 30
}
config = RvpnseConfig.from_dict(config_dict)

client = RvpnseClient(config)
```

### 3. Synchronous API

```python
from rvpnse import RvpnseClient, RvpnseConfig

def sync_vpn_example():
    config = RvpnseConfig(
        server_host="vpn.example.com",
        server_port=443,
        username="user",
        password="pass",
        hub_name="VPN"
    )
    
    client = RvpnseClient(config)
    
    try:
        # Synchronous connection
        client.connect_sync()
        print("Connected!")
        
        # Check status
        status = client.get_status()
        print(f"Status: {status.state}")
        print(f"Server IP: {status.server_ip}")
        print(f"Local IP: {status.local_ip}")
        
    finally:
        client.disconnect_sync()

sync_vpn_example()
```

## 🔧 Advanced Configuration

### 1. Comprehensive Configuration Class

```python
from dataclasses import dataclass
from typing import Optional, Dict, Any
from rvpnse import RvpnseConfig

@dataclass
class AdvancedVpnConfig:
    # Server settings
    server_host: str
    server_port: int = 443
    hub_name: str = "VPN"
    
    # Authentication
    username: str
    password: Optional[str] = None
    certificate_file: Optional[str] = None
    private_key_file: Optional[str] = None
    
    # Connection settings
    enable_compression: bool = True
    keep_alive_interval: int = 30
    connection_timeout: int = 10
    retry_attempts: int = 3
    retry_delay: int = 5
    
    # Security settings
    verify_server_cert: bool = True
    custom_ca_cert: Optional[str] = None
    min_tls_version: str = "1.2"
    cipher_suites: Optional[list] = None
    
    # Performance settings
    tcp_nodelay: bool = True
    socket_buffer_size: int = 65536
    max_concurrent_connections: int = 1
    
    # Logging
    enable_logging: bool = False
    log_level: str = "INFO"
    log_file: Optional[str] = None
    
    def to_rvpnse_config(self) -> RvpnseConfig:
        """Convert to RvpnseConfig object."""
        return RvpnseConfig(
            server_host=self.server_host,
            server_port=self.server_port,
            username=self.username,
            password=self.password,
            hub_name=self.hub_name,
            enable_compression=self.enable_compression,
            keep_alive_interval=self.keep_alive_interval,
            connection_timeout=self.connection_timeout,
            verify_server_cert=self.verify_server_cert,
            custom_ca_cert=self.custom_ca_cert,
            enable_logging=self.enable_logging,
            log_level=self.log_level
        )
    
    @classmethod
    def from_file(cls, file_path: str) -> 'AdvancedVpnConfig':
        """Load configuration from TOML file."""
        import toml
        with open(file_path, 'r') as f:
            data = toml.load(f)
        return cls(**data)
    
    @classmethod
    def from_env(cls) -> 'AdvancedVpnConfig':
        """Load configuration from environment variables."""
        import os
        return cls(
            server_host=os.environ['VPN_SERVER_HOST'],
            server_port=int(os.environ.get('VPN_SERVER_PORT', 443)),
            username=os.environ['VPN_USERNAME'],
            password=os.environ.get('VPN_PASSWORD'),
            hub_name=os.environ.get('VPN_HUB_NAME', 'VPN'),
            enable_compression=os.environ.get('VPN_COMPRESSION', 'true').lower() == 'true',
            enable_logging=os.environ.get('VPN_LOGGING', 'false').lower() == 'true'
        )
```

### 2. Connection State Management

```python
import asyncio
from enum import Enum
from typing import Callable, Optional
from rvpnse import RvpnseClient, ConnectionState

class VpnManager:
    def __init__(self, config: AdvancedVpnConfig):
        self.config = config
        self.client = RvpnseClient(config.to_rvpnse_config())
        self.state_callbacks: Dict[ConnectionState, List[Callable]] = {}
        self.auto_reconnect = True
        self.reconnect_task: Optional[asyncio.Task] = None
        
    def add_state_callback(self, state: ConnectionState, callback: Callable):
        """Add callback for state changes."""
        if state not in self.state_callbacks:
            self.state_callbacks[state] = []
        self.state_callbacks[state].append(callback)
    
    async def connect_with_retry(self) -> bool:
        """Connect with automatic retry logic."""
        for attempt in range(self.config.retry_attempts):
            try:
                await self.client.connect()
                self._trigger_callbacks(ConnectionState.CONNECTED)
                return True
                
            except Exception as e:
                print(f"Connection attempt {attempt + 1} failed: {e}")
                if attempt < self.config.retry_attempts - 1:
                    await asyncio.sleep(self.config.retry_delay)
                else:
                    self._trigger_callbacks(ConnectionState.ERROR)
                    return False
        
        return False
    
    async def disconnect(self):
        """Disconnect and stop auto-reconnect."""
        self.auto_reconnect = False
        if self.reconnect_task:
            self.reconnect_task.cancel()
        
        await self.client.disconnect()
        self._trigger_callbacks(ConnectionState.DISCONNECTED)
    
    async def start_monitoring(self):
        """Start connection monitoring with auto-reconnect."""
        while self.auto_reconnect:
            try:
                status = await self.client.get_status()
                
                if status.state == ConnectionState.DISCONNECTED and self.auto_reconnect:
                    print("Connection lost, attempting to reconnect...")
                    await self.connect_with_retry()
                
                await asyncio.sleep(5)  # Check every 5 seconds
                
            except Exception as e:
                print(f"Monitoring error: {e}")
                await asyncio.sleep(10)
    
    def _trigger_callbacks(self, state: ConnectionState):
        """Trigger callbacks for state change."""
        if state in self.state_callbacks:
            for callback in self.state_callbacks[state]:
                try:
                    callback()
                except Exception as e:
                    print(f"Callback error: {e}")

# Usage example
async def managed_vpn_example():
    config = AdvancedVpnConfig(
        server_host="vpn.example.com",
        username="user",
        password="pass",
        retry_attempts=5,
        retry_delay=3
    )
    
    manager = VpnManager(config)
    
    # Add callbacks
    manager.add_state_callback(
        ConnectionState.CONNECTED, 
        lambda: print("🟢 VPN Connected!")
    )
    manager.add_state_callback(
        ConnectionState.DISCONNECTED, 
        lambda: print("🔴 VPN Disconnected!")
    )
    
    # Connect and start monitoring
    await manager.connect_with_retry()
    
    # Start monitoring in background
    monitoring_task = asyncio.create_task(manager.start_monitoring())
    
    try:
        # Keep running
        await asyncio.sleep(300)  # 5 minutes
    finally:
        await manager.disconnect()
        monitoring_task.cancel()

asyncio.run(managed_vpn_example())
```

## 📊 Performance Monitoring and Statistics

### 1. Real-time Statistics

```python
import asyncio
import time
from dataclasses import dataclass
from typing import List
from rvpnse import RvpnseClient

@dataclass
class NetworkStats:
    timestamp: float
    bytes_sent: int
    bytes_received: int
    packets_sent: int
    packets_received: int
    latency_ms: float
    packet_loss_percent: float

class VpnStatisticsMonitor:
    def __init__(self, client: RvpnseClient):
        self.client = client
        self.stats_history: List[NetworkStats] = []
        self.monitoring = False
    
    async def start_monitoring(self, interval: float = 1.0):
        """Start collecting statistics."""
        self.monitoring = True
        
        while self.monitoring:
            try:
                stats = await self.client.get_detailed_stats()
                
                network_stats = NetworkStats(
                    timestamp=time.time(),
                    bytes_sent=stats.bytes_sent,
                    bytes_received=stats.bytes_received,
                    packets_sent=stats.packets_sent,
                    packets_received=stats.packets_received,
                    latency_ms=stats.latency_ms,
                    packet_loss_percent=stats.packet_loss_percent
                )
                
                self.stats_history.append(network_stats)
                
                # Keep only last 1000 entries
                if len(self.stats_history) > 1000:
                    self.stats_history.pop(0)
                
                # Print current stats
                self.print_current_stats(network_stats)
                
                await asyncio.sleep(interval)
                
            except Exception as e:
                print(f"Statistics monitoring error: {e}")
                await asyncio.sleep(interval)
    
    def stop_monitoring(self):
        """Stop collecting statistics."""
        self.monitoring = False
    
    def print_current_stats(self, stats: NetworkStats):
        """Print current network statistics."""
        throughput_mbps = self.calculate_throughput_mbps()
        
        print(f"📊 VPN Statistics:")
        print(f"   Throughput: {throughput_mbps:.2f} Mbps")
        print(f"   Latency: {stats.latency_ms:.1f}ms")
        print(f"   Packet Loss: {stats.packet_loss_percent:.1f}%")
        print(f"   Bytes: ↓{self.format_bytes(stats.bytes_received)} "
              f"↑{self.format_bytes(stats.bytes_sent)}")
        print("-" * 40)
    
    def calculate_throughput_mbps(self) -> float:
        """Calculate average throughput over last 10 seconds."""
        if len(self.stats_history) < 2:
            return 0.0
        
        # Get stats from 10 seconds ago
        cutoff_time = time.time() - 10
        recent_stats = [s for s in self.stats_history if s.timestamp >= cutoff_time]
        
        if len(recent_stats) < 2:
            return 0.0
        
        oldest = recent_stats[0]
        newest = recent_stats[-1]
        
        time_diff = newest.timestamp - oldest.timestamp
        bytes_diff = (newest.bytes_sent + newest.bytes_received) - \
                    (oldest.bytes_sent + oldest.bytes_received)
        
        if time_diff <= 0:
            return 0.0
        
        bytes_per_second = bytes_diff / time_diff
        return (bytes_per_second * 8) / (1024 * 1024)  # Convert to Mbps
    
    def format_bytes(self, bytes_count: int) -> str:
        """Format bytes in human-readable format."""
        for unit in ['B', 'KB', 'MB', 'GB']:
            if bytes_count < 1024:
                return f"{bytes_count:.1f}{unit}"
            bytes_count /= 1024
        return f"{bytes_count:.1f}TB"
    
    def get_average_latency(self, seconds: int = 60) -> float:
        """Get average latency over specified seconds."""
        cutoff_time = time.time() - seconds
        recent_stats = [s for s in self.stats_history if s.timestamp >= cutoff_time]
        
        if not recent_stats:
            return 0.0
        
        return sum(s.latency_ms for s in recent_stats) / len(recent_stats)
    
    def get_connection_quality(self) -> str:
        """Assess connection quality based on latency and packet loss."""
        avg_latency = self.get_average_latency(30)  # Last 30 seconds
        
        if not self.stats_history:
            return "Unknown"
        
        latest_loss = self.stats_history[-1].packet_loss_percent
        
        if avg_latency < 50 and latest_loss < 1:
            return "Excellent"
        elif avg_latency < 100 and latest_loss < 2:
            return "Good"
        elif avg_latency < 200 and latest_loss < 5:
            return "Fair"
        else:
            return "Poor"

# Usage example
async def monitoring_example():
    config = RvpnseConfig(
        server_host="vpn.example.com",
        username="user",
        password="pass",
        hub_name="VPN"
    )
    
    client = RvpnseClient(config)
    monitor = VpnStatisticsMonitor(client)
    
    try:
        await client.connect()
        
        # Start monitoring in background
        monitor_task = asyncio.create_task(monitor.start_monitoring(2.0))
        
        # Run for 2 minutes
        await asyncio.sleep(120)
        
        print(f"Final connection quality: {monitor.get_connection_quality()}")
        
    finally:
        monitor.stop_monitoring()
        await client.disconnect()

asyncio.run(monitoring_example())
```

## 🔒 Security and Authentication

### 1. Certificate-based Authentication

```python
import ssl
from pathlib import Path
from rvpnse import RvpnseConfig, RvpnseClient

class SecureVpnClient:
    def __init__(self, server_host: str, username: str):
        self.server_host = server_host
        self.username = username
        self.client: Optional[RvpnseClient] = None
    
    @classmethod
    def from_certificate(cls, server_host: str, username: str, 
                        cert_file: str, key_file: str, ca_file: str = None):
        """Create client with certificate authentication."""
        instance = cls(server_host, username)
        
        config = RvpnseConfig(
            server_host=server_host,
            username=username,
            auth_method="certificate",
            client_cert_file=cert_file,
            client_key_file=key_file,
            ca_cert_file=ca_file,
            verify_server_cert=True
        )
        
        instance.client = RvpnseClient(config)
        return instance
    
    @classmethod
    def from_keystore(cls, server_host: str, username: str, 
                     keystore_file: str, keystore_password: str):
        """Create client with PKCS#12 keystore."""
        instance = cls(server_host, username)
        
        config = RvpnseConfig(
            server_host=server_host,
            username=username,
            auth_method="keystore",
            keystore_file=keystore_file,
            keystore_password=keystore_password,
            verify_server_cert=True
        )
        
        instance.client = RvpnseClient(config)
        return instance
    
    def validate_certificate_chain(self, cert_data: bytes) -> bool:
        """Validate certificate chain."""
        try:
            # Load certificate
            cert = ssl.PEM_cert_to_DER_cert(cert_data.decode())
            
            # Perform custom validation
            # - Check expiration
            # - Verify issuer
            # - Check against CRL
            
            return True
        except Exception as e:
            print(f"Certificate validation failed: {e}")
            return False
    
    async def connect_secure(self) -> bool:
        """Connect with enhanced security checks."""
        if not self.client:
            raise ValueError("Client not configured")
        
        try:
            # Set certificate validation callback
            self.client.set_cert_validator(self.validate_certificate_chain)
            
            # Connect
            await self.client.connect()
            
            # Verify connection security
            security_info = await self.client.get_security_info()
            print(f"TLS Version: {security_info.tls_version}")
            print(f"Cipher Suite: {security_info.cipher_suite}")
            print(f"Server Certificate Valid: {security_info.cert_valid}")
            
            return True
            
        except Exception as e:
            print(f"Secure connection failed: {e}")
            return False

# Usage example
async def secure_connection_example():
    # Certificate-based authentication
    client = SecureVpnClient.from_certificate(
        server_host="secure-vpn.example.com",
        username="cert_user",
        cert_file="client.crt",
        key_file="client.key",
        ca_file="ca.crt"
    )
    
    await client.connect_secure()
```

### 2. Credential Management

```python
import keyring
import getpass
from cryptography.fernet import Fernet
from typing import Optional

class CredentialManager:
    def __init__(self, service_name: str = "rvpnse-vpn"):
        self.service_name = service_name
        self.encryption_key = self._get_or_create_key()
    
    def _get_or_create_key(self) -> bytes:
        """Get or create encryption key for local storage."""
        key = keyring.get_password(self.service_name, "encryption_key")
        if not key:
            key = Fernet.generate_key().decode()
            keyring.set_password(self.service_name, "encryption_key", key)
        return key.encode()
    
    def store_credentials(self, username: str, password: str) -> bool:
        """Store credentials securely."""
        try:
            fernet = Fernet(self.encryption_key)
            encrypted_password = fernet.encrypt(password.encode())
            
            # Store in system keyring
            keyring.set_password(self.service_name, username, encrypted_password.decode())
            return True
            
        except Exception as e:
            print(f"Failed to store credentials: {e}")
            return False
    
    def get_credentials(self, username: str) -> Optional[str]:
        """Retrieve credentials securely."""
        try:
            encrypted_password = keyring.get_password(self.service_name, username)
            if not encrypted_password:
                return None
            
            fernet = Fernet(self.encryption_key)
            password = fernet.decrypt(encrypted_password.encode()).decode()
            return password
            
        except Exception as e:
            print(f"Failed to retrieve credentials: {e}")
            return None
    
    def delete_credentials(self, username: str) -> bool:
        """Delete stored credentials."""
        try:
            keyring.delete_password(self.service_name, username)
            return True
        except Exception as e:
            print(f"Failed to delete credentials: {e}")
            return False
    
    def prompt_credentials(self, username: str = None) -> tuple[str, str]:
        """Prompt user for credentials."""
        if not username:
            username = input("Username: ")
        
        # Try to get stored password first
        password = self.get_credentials(username)
        if password:
            use_stored = input(f"Use stored password for {username}? (y/n): ")
            if use_stored.lower() == 'y':
                return username, password
        
        # Prompt for password
        password = getpass.getpass(f"Password for {username}: ")
        
        # Ask to store
        store = input("Store password securely? (y/n): ")
        if store.lower() == 'y':
            self.store_credentials(username, password)
        
        return username, password

# Usage example
async def credential_management_example():
    cred_manager = CredentialManager()
    
    # Get credentials (prompt if not stored)
    username, password = cred_manager.prompt_credentials()
    
    config = RvpnseConfig(
        server_host="vpn.example.com",
        username=username,
        password=password,
        hub_name="VPN"
    )
    
    client = RvpnseClient(config)
    await client.connect()
```

## 🧪 Testing and Quality Assurance

### 1. Unit Testing

```python
import pytest
import asyncio
from unittest.mock import AsyncMock, patch
from rvpnse import RvpnseClient, RvpnseConfig, ConnectionState

class TestRvpnseClient:
    @pytest.fixture
    def mock_config(self):
        return RvpnseConfig(
            server_host="test.example.com",
            server_port=443,
            username="testuser",
            password="testpass",
            hub_name="TEST"
        )
    
    @pytest.fixture
    def client(self, mock_config):
        return RvpnseClient(mock_config)
    
    @pytest.mark.asyncio
    async def test_successful_connection(self, client):
        """Test successful VPN connection."""
        with patch.object(client, '_native_connect', new_callable=AsyncMock) as mock_connect:
            mock_connect.return_value = True
            
            result = await client.connect()
            assert result is True
            mock_connect.assert_called_once()
    
    @pytest.mark.asyncio
    async def test_connection_failure(self, client):
        """Test connection failure handling."""
        with patch.object(client, '_native_connect', new_callable=AsyncMock) as mock_connect:
            mock_connect.side_effect = Exception("Connection failed")
            
            with pytest.raises(Exception):
                await client.connect()
    
    @pytest.mark.asyncio
    async def test_status_monitoring(self, client):
        """Test status monitoring functionality."""
        with patch.object(client, 'get_status', new_callable=AsyncMock) as mock_status:
            mock_status.return_value.state = ConnectionState.CONNECTED
            mock_status.return_value.server_ip = "192.168.1.1"
            
            status = await client.get_status()
            assert status.state == ConnectionState.CONNECTED
            assert status.server_ip == "192.168.1.1"
    
    def test_config_validation(self):
        """Test configuration validation."""
        # Valid config
        valid_config = RvpnseConfig(
            server_host="valid.example.com",
            username="user",
            password="pass"
        )
        assert valid_config.is_valid()
        
        # Invalid config (missing required fields)
        with pytest.raises(ValueError):
            RvpnseConfig(server_host="", username="", password="")

class TestAdvancedFeatures:
    @pytest.mark.asyncio
    async def test_retry_logic(self):
        """Test connection retry logic."""
        config = AdvancedVpnConfig(
            server_host="test.example.com",
            username="user",
            password="pass",
            retry_attempts=3,
            retry_delay=0.1  # Fast retry for testing
        )
        
        manager = VpnManager(config)
        
        with patch.object(manager.client, 'connect', new_callable=AsyncMock) as mock_connect:
            # Fail first two attempts, succeed on third
            mock_connect.side_effect = [
                Exception("First attempt failed"),
                Exception("Second attempt failed"),
                None  # Success
            ]
            
            result = await manager.connect_with_retry()
            assert result is True
            assert mock_connect.call_count == 3
    
    @pytest.mark.asyncio
    async def test_statistics_collection(self):
        """Test statistics collection."""
        client = RvpnseClient(RvpnseConfig(
            server_host="test.example.com",
            username="user",
            password="pass"
        ))
        
        monitor = VpnStatisticsMonitor(client)
        
        with patch.object(client, 'get_detailed_stats', new_callable=AsyncMock) as mock_stats:
            mock_stats.return_value.bytes_sent = 1000
            mock_stats.return_value.bytes_received = 2000
            mock_stats.return_value.latency_ms = 50.0
            
            # Start monitoring briefly
            monitor_task = asyncio.create_task(monitor.start_monitoring(0.1))
            await asyncio.sleep(0.3)  # Let it collect a few samples
            monitor.stop_monitoring()
            
            # Check that statistics were collected
            assert len(monitor.stats_history) > 0
            assert monitor.stats_history[-1].latency_ms == 50.0

# Integration tests
class TestIntegration:
    @pytest.mark.integration
    @pytest.mark.asyncio
    async def test_real_connection(self):
        """Test with real VPN server (requires test server)."""
        # Skip if no test server configured
        test_server = os.environ.get('TEST_VPN_SERVER')
        if not test_server:
            pytest.skip("No test VPN server configured")
        
        config = RvpnseConfig(
            server_host=test_server,
            username=os.environ.get('TEST_VPN_USERNAME'),
            password=os.environ.get('TEST_VPN_PASSWORD'),
            hub_name=os.environ.get('TEST_VPN_HUB', 'VPN')
        )
        
        client = RvpnseClient(config)
        
        try:
            await client.connect()
            
            # Verify connection
            status = await client.get_status()
            assert status.state == ConnectionState.CONNECTED
            
            # Test data transfer
            stats_before = await client.get_detailed_stats()
            await asyncio.sleep(1)  # Let some data flow
            stats_after = await client.get_detailed_stats()
            
            # Should have some network activity
            assert stats_after.bytes_sent >= stats_before.bytes_sent
            
        finally:
            await client.disconnect()

# Run tests
if __name__ == "__main__":
    pytest.main([__file__, "-v"])
```

### 2. Performance Testing

```python
import asyncio
import time
import statistics
from concurrent.futures import ThreadPoolExecutor
from rvpnse import RvpnseClient, RvpnseConfig

class PerformanceTestSuite:
    def __init__(self, config: RvpnseConfig):
        self.config = config
        self.results = {}
    
    async def test_connection_latency(self, iterations: int = 10):
        """Test connection establishment latency."""
        latencies = []
        
        for i in range(iterations):
            client = RvpnseClient(self.config)
            
            start_time = time.time()
            await client.connect()
            end_time = time.time()
            
            latency = (end_time - start_time) * 1000  # Convert to ms
            latencies.append(latency)
            
            await client.disconnect()
            await asyncio.sleep(1)  # Cool down between tests
        
        self.results['connection_latency'] = {
            'mean': statistics.mean(latencies),
            'median': statistics.median(latencies),
            'std_dev': statistics.stdev(latencies) if len(latencies) > 1 else 0,
            'min': min(latencies),
            'max': max(latencies)
        }
        
        print(f"Connection Latency Test Results:")
        print(f"  Mean: {self.results['connection_latency']['mean']:.2f}ms")
        print(f"  Median: {self.results['connection_latency']['median']:.2f}ms")
        print(f"  Std Dev: {self.results['connection_latency']['std_dev']:.2f}ms")
    
    async def test_throughput(self, duration: int = 30):
        """Test network throughput."""
        client = RvpnseClient(self.config)
        
        try:
            await client.connect()
            
            # Wait for connection to stabilize
            await asyncio.sleep(2)
            
            initial_stats = await client.get_detailed_stats()
            start_time = time.time()
            
            # Generate network traffic (implement traffic generation)
            await self._generate_traffic(client, duration)
            
            end_time = time.time()
            final_stats = await client.get_detailed_stats()
            
            # Calculate throughput
            time_elapsed = end_time - start_time
            bytes_transferred = (final_stats.bytes_sent + final_stats.bytes_received) - \
                              (initial_stats.bytes_sent + initial_stats.bytes_received)
            
            throughput_mbps = (bytes_transferred * 8) / (time_elapsed * 1024 * 1024)
            
            self.results['throughput'] = {
                'mbps': throughput_mbps,
                'bytes_transferred': bytes_transferred,
                'duration': time_elapsed
            }
            
            print(f"Throughput Test Results:")
            print(f"  Throughput: {throughput_mbps:.2f} Mbps")
            print(f"  Bytes Transferred: {bytes_transferred:,}")
            print(f"  Duration: {time_elapsed:.1f}s")
            
        finally:
            await client.disconnect()
    
    async def test_concurrent_connections(self, num_connections: int = 5):
        """Test multiple concurrent connections."""
        async def create_connection():
            client = RvpnseClient(self.config)
            start_time = time.time()
            
            try:
                await client.connect()
                connect_time = time.time() - start_time
                
                # Keep connection alive briefly
                await asyncio.sleep(5)
                
                return {'success': True, 'connect_time': connect_time}
            except Exception as e:
                return {'success': False, 'error': str(e)}
            finally:
                await client.disconnect()
        
        start_time = time.time()
        tasks = [create_connection() for _ in range(num_connections)]
        results = await asyncio.gather(*tasks)
        total_time = time.time() - start_time
        
        successful = [r for r in results if r['success']]
        failed = [r for r in results if not r['success']]
        
        self.results['concurrent_connections'] = {
            'total_connections': num_connections,
            'successful': len(successful),
            'failed': len(failed),
            'success_rate': len(successful) / num_connections * 100,
            'total_time': total_time,
            'avg_connect_time': statistics.mean([r['connect_time'] for r in successful]) if successful else 0
        }
        
        print(f"Concurrent Connections Test Results:")
        print(f"  Success Rate: {self.results['concurrent_connections']['success_rate']:.1f}%")
        print(f"  Avg Connect Time: {self.results['concurrent_connections']['avg_connect_time']:.2f}s")
    
    async def _generate_traffic(self, client: RvpnseClient, duration: int):
        """Generate network traffic for throughput testing."""
        # Implementation depends on your traffic generation method
        # This could involve HTTP requests, data uploads, etc.
        await asyncio.sleep(duration)

# Usage example
async def run_performance_tests():
    config = RvpnseConfig(
        server_host=os.environ.get('PERF_TEST_SERVER', 'vpn.example.com'),
        username=os.environ.get('PERF_TEST_USERNAME'),
        password=os.environ.get('PERF_TEST_PASSWORD'),
        hub_name="VPN"
    )
    
    test_suite = PerformanceTestSuite(config)
    
    print("Running performance tests...")
    await test_suite.test_connection_latency(5)
    await test_suite.test_throughput(15)
    await test_suite.test_concurrent_connections(3)
    
    print("\nAll tests completed!")

if __name__ == "__main__":
    asyncio.run(run_performance_tests())
```

## 🚀 Production Deployment

### 1. Systemd Service (Linux)

```python
#!/usr/bin/env python3
# /usr/local/bin/rvpnse-daemon

import asyncio
import signal
import sys
import logging
from pathlib import Path
from rvpnse import RvpnseClient, RvpnseConfig

class VpnDaemon:
    def __init__(self, config_file: str):
        self.config_file = config_file
        self.client = None
        self.running = False
        
        # Setup logging
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            handlers=[
                logging.StreamHandler(sys.stdout),
                logging.FileHandler('/var/log/rvpnse.log')
            ]
        )
        self.logger = logging.getLogger('rvpnse-daemon')
    
    async def start(self):
        """Start the VPN daemon."""
        self.logger.info("Starting rVPNSE daemon")
        
        try:
            # Load configuration
            config = RvpnseConfig.from_file(self.config_file)
            self.client = RvpnseClient(config)
            
            # Setup signal handlers
            signal.signal(signal.SIGTERM, self._signal_handler)
            signal.signal(signal.SIGINT, self._signal_handler)
            
            self.running = True
            
            # Main daemon loop
            while self.running:
                try:
                    if not self.client.is_connected():
                        self.logger.info("Connecting to VPN...")
                        await self.client.connect()
                        self.logger.info("VPN connected successfully")
                    
                    # Health check
                    await self._health_check()
                    
                    await asyncio.sleep(10)  # Check every 10 seconds
                    
                except Exception as e:
                    self.logger.error(f"Error in daemon loop: {e}")
                    await asyncio.sleep(30)  # Wait before retry
            
        except Exception as e:
            self.logger.error(f"Failed to start daemon: {e}")
            sys.exit(1)
        finally:
            await self.stop()
    
    async def stop(self):
        """Stop the VPN daemon."""
        self.logger.info("Stopping rVPNSE daemon")
        self.running = False
        
        if self.client:
            try:
                await self.client.disconnect()
                self.logger.info("VPN disconnected")
            except Exception as e:
                self.logger.error(f"Error disconnecting: {e}")
    
    async def _health_check(self):
        """Perform health check on VPN connection."""
        try:
            stats = await self.client.get_detailed_stats()
            
            # Check for connection issues
            if stats.packet_loss_percent > 10:
                self.logger.warning(f"High packet loss detected: {stats.packet_loss_percent}%")
            
            if stats.latency_ms > 1000:
                self.logger.warning(f"High latency detected: {stats.latency_ms}ms")
                
        except Exception as e:
            self.logger.error(f"Health check failed: {e}")
    
    def _signal_handler(self, signum, frame):
        """Handle system signals."""
        self.logger.info(f"Received signal {signum}, shutting down...")
        self.running = False

async def main():
    if len(sys.argv) != 3 or sys.argv[1] != '--config':
        print("Usage: rvpnse-daemon --config <config_file>")
        sys.exit(1)
    
    config_file = sys.argv[2]
    
    if not Path(config_file).exists():
        print(f"Configuration file not found: {config_file}")
        sys.exit(1)
    
    daemon = VpnDaemon(config_file)
    await daemon.start()

if __name__ == "__main__":
    asyncio.run(main())
```

### 2. Docker Container

```dockerfile
# Dockerfile
FROM python:3.11-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    libc6-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd --create-home --shell /bin/bash rvpnse

# Set working directory
WORKDIR /app

# Copy requirements and install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY . .

# Create necessary directories
RUN mkdir -p /var/log/rvpnse /etc/rvpnse

# Set ownership
RUN chown -R rvpnse:rvpnse /app /var/log/rvpnse /etc/rvpnse

# Switch to app user
USER rvpnse

# Expose health check port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD python -c "import requests; requests.get('http://localhost:8080/health')"

# Start the daemon
CMD ["python", "-m", "rvpnse_daemon", "--config", "/etc/rvpnse/config.toml"]
```

```yaml
# docker-compose.yml
version: '3.8'

services:
  rvpnse:
    build: .
    restart: unless-stopped
    volumes:
      - ./config:/etc/rvpnse:ro
      - rvpnse_logs:/var/log/rvpnse
    environment:
      - VPN_SERVER_HOST=${VPN_SERVER_HOST}
      - VPN_USERNAME=${VPN_USERNAME}
      - VPN_PASSWORD=${VPN_PASSWORD}
    networks:
      - vpn_network
    cap_add:
      - NET_ADMIN
    devices:
      - /dev/net/tun
    sysctls:
      - net.ipv4.ip_forward=1

volumes:
  rvpnse_logs:

networks:
  vpn_network:
    driver: bridge
```

## 🆘 Troubleshooting

### Common Issues and Solutions

| Issue | Symptoms | Solution |
|-------|----------|----------|
| **Import Error** | `ModuleNotFoundError: No module named 'rvpnse'` | Install with `pip install rvpnse` |
| **Native Library Not Found** | `OSError: cannot load library` | Check platform-specific library installation |
| **Permission Denied** | Connection fails with permission error | Run with appropriate privileges or configure capabilities |
| **Connection Timeout** | Hangs during connection | Check network connectivity and firewall settings |
| **Certificate Errors** | SSL/TLS verification failures | Verify server certificates and CA configuration |
| **Memory Leaks** | Increasing memory usage | Ensure proper cleanup of client objects |

### Debug Logging

```python
import logging
from rvpnse import RvpnseClient, RvpnseConfig

# Enable debug logging
logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger('rvpnse')

# Create client with debug logging
config = RvpnseConfig(
    server_host="vpn.example.com",
    username="user",
    password="pass",
    enable_logging=True,
    log_level="DEBUG"
)

client = RvpnseClient(config)
```

## 📚 Next Steps

1. **Install rVPNSE Python package** using pip
2. **Configure your VPN settings** using the configuration examples
3. **Implement basic connection logic** using our async/sync examples
4. **Add error handling and retry logic** for production robustness
5. **Implement monitoring and statistics** for operational visibility
6. **Deploy using systemd/Docker** for production environments

**Need help?** Check our [Python-specific troubleshooting](../../07-troubleshooting/python.md) or [join our community](https://github.com/devstroop/rvpnse/discussions).
