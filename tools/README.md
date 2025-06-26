# Development Tools

This directory contains various tools and scripts for building and maintaining rVPNSE:

## Build Tools

- **`build.py`** - Main build script for cross-platform compilation
- **`docker/`** - Docker-related files for containerized builds

## Utility Scripts

- **`update_project_name.sh`** - Script for updating project name references throughout the codebase

## Docker

The `docker/` subdirectory contains:
- **`Dockerfile.android`** - Docker configuration for Android cross-compilation

## Usage

### Building for Different Platforms

```bash
# Build for current platform
python3 tools/build.py

# Build for Android (using Docker)
docker build -f tools/docker/Dockerfile.android .
```

### Project Maintenance

```bash
# Update project name references
./tools/update_project_name.sh old_name new_name
```

See the [build documentation](../docs/05-build/) for more detailed build instructions.
