# Configuration Files

This directory contains various configuration files for rVPNSE:

## Files

- **`config-mobile.toml`** - Configuration optimized for mobile devices
- **`config-production.toml`** - Production environment configuration
- **`example.toml`** - Example configuration file with all available options
- **`test_config.toml`** - Configuration for testing scenarios

## Usage

Copy one of these files to your working directory and modify as needed:

```bash
# For mobile development
cp config/config-mobile.toml my-app-config.toml

# For production deployment
cp config/config-production.toml vpn-config.toml

# For getting started (with examples)
cp config/example.toml config.toml
```

## Security Note

- Never commit actual credentials or private keys to version control
- Use environment variables for sensitive configuration values
- The `config/secrets/` directory is ignored by Git for this purpose
