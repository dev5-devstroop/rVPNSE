#!/usr/bin/env python3
"""
rVPNSE Unified Build System
Production-ready cross-platform builder for rVPNSE VPN library
"""

import os
import sys
import json
import shutil
import subprocess
import argparse
import platform
import tempfile
from pathlib import Path
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass
from enum import Enum
import logging

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class BuildMode(Enum):
    DEBUG = "debug"
    RELEASE = "release"

class Platform(Enum):
    LINUX = "linux"
    MACOS = "macos"
    WINDOWS = "windows"
    ANDROID = "android"
    IOS = "ios"

@dataclass
class Target:
    name: str
    rust_target: str
    platform: Platform
    arch: str
    android_arch: Optional[str] = None
    min_api: Optional[int] = None

# Production target configurations
TARGETS = {
    # Desktop targets
    "linux-x64": Target("linux-x64", "x86_64-unknown-linux-gnu", Platform.LINUX, "x64"),
    "macos-x64": Target("macos-x64", "x86_64-apple-darwin", Platform.MACOS, "x64"),
    "macos-arm64": Target("macos-arm64", "aarch64-apple-darwin", Platform.MACOS, "arm64"),
    "windows-x64": Target("windows-x64", "x86_64-pc-windows-msvc", Platform.WINDOWS, "x64"),
    
    # Android targets
    "android-arm64": Target("android-arm64", "aarch64-linux-android", Platform.ANDROID, "arm64", "arm64-v8a", 21),
    "android-arm": Target("android-arm", "armv7-linux-androideabi", Platform.ANDROID, "arm", "armeabi-v7a", 19),
    "android-x64": Target("android-x64", "x86_64-linux-android", Platform.ANDROID, "x64", "x86_64", 21),
    "android-x86": Target("android-x86", "i686-linux-android", Platform.ANDROID, "x86", "x86", 19),
    
    # iOS targets
    "ios-arm64": Target("ios-arm64", "aarch64-apple-ios", Platform.IOS, "arm64"),
    "ios-x64-sim": Target("ios-x64-sim", "x86_64-apple-ios", Platform.IOS, "x64"),
    "ios-arm64-sim": Target("ios-arm64-sim", "aarch64-apple-ios-sim", Platform.IOS, "arm64"),
}

class rVPNSEBuilder:
    def __init__(self, project_root: Path, mode: BuildMode = BuildMode.RELEASE):
        self.project_root = project_root
        self.mode = mode
        self.build_dir = project_root / "target"
        self.output_dir = project_root / "dist"
        self.android_ndk_root = self._find_android_ndk()
        
        # Ensure we're in the right directory
        if not (project_root / "Cargo.toml").exists():
            raise RuntimeError(f"Cargo.toml not found in {project_root}")
    
    def _find_android_ndk(self) -> Optional[Path]:
        """Find Android NDK installation"""
        possible_paths = [
            Path.home() / "Android" / "Sdk" / "ndk" / "25.2.9519653",
            Path.home() / "Library" / "Android" / "sdk" / "ndk" / "25.2.9519653",
            Path("/usr/local/android-ndk-r25c"),
            Path("/opt/android-ndk-r25c"),
        ]
        
        # Check environment variable first
        ndk_root = os.environ.get("ANDROID_NDK_ROOT")
        if ndk_root:
            ndk_path = Path(ndk_root)
            if ndk_path.exists():
                return ndk_path
        
        # Check common locations
        for path in possible_paths:
            if path.exists() and (path / "toolchains" / "llvm" / "prebuilt").exists():
                logger.info(f"Found Android NDK at: {path}")
                return path
        
        logger.warning("Android NDK not found. Android builds will be skipped.")
        return None
    
    def _run_command(self, cmd: List[str], cwd: Optional[Path] = None, env: Optional[Dict] = None) -> bool:
        """Run a command and return success status"""
        try:
            logger.info(f"Running: {' '.join(cmd)}")
            result = subprocess.run(
                cmd,
                cwd=cwd or self.project_root,
                env=env or os.environ.copy(),
                capture_output=True,
                text=True
            )
            
            if result.returncode != 0:
                logger.error(f"Command failed with code {result.returncode}")
                logger.error(f"STDOUT: {result.stdout}")
                logger.error(f"STDERR: {result.stderr}")
                return False
            
            logger.debug(f"STDOUT: {result.stdout}")
            return True
            
        except Exception as e:
            logger.error(f"Command execution failed: {e}")
            return False
    
    def _setup_rust_environment(self) -> bool:
        """Ensure Rust is properly set up"""
        logger.info("Setting up Rust environment...")
        
        # Check if cargo is available
        if not shutil.which("cargo"):
            logger.error("Cargo not found. Please install Rust.")
            return False
        
        # Install required targets
        all_targets = [target.rust_target for target in TARGETS.values()]
        for rust_target in all_targets:
            if not self._run_command(["rustup", "target", "add", rust_target]):
                logger.warning(f"Failed to add target {rust_target}")
        
        return True
    
    def _setup_android_environment(self, target: Target) -> Dict[str, str]:
        """Set up Android build environment"""
        if not self.android_ndk_root:
            raise RuntimeError("Android NDK not found")
        
        # Determine toolchain directory
        system = platform.system().lower()
        if system == "darwin":
            toolchain_dir = self.android_ndk_root / "toolchains" / "llvm" / "prebuilt" / "darwin-x86_64"
        else:
            toolchain_dir = self.android_ndk_root / "toolchains" / "llvm" / "prebuilt" / "linux-x86_64"
        
        if not toolchain_dir.exists():
            raise RuntimeError(f"Android toolchain not found: {toolchain_dir}")
        
        # Set up clang target
        clang_targets = {
            "aarch64-linux-android": f"aarch64-linux-android{target.min_api}",
            "armv7-linux-androideabi": f"armv7a-linux-androideabi{target.min_api}",
            "x86_64-linux-android": f"x86_64-linux-android{target.min_api}",
            "i686-linux-android": f"i686-linux-android{target.min_api}",
        }
        
        clang_target = clang_targets[target.rust_target]
        target_upper = target.rust_target.upper().replace("-", "_")
        
        env = os.environ.copy()
        env.update({
            "ANDROID_NDK_ROOT": str(self.android_ndk_root),
            "ANDROID_NDK_HOME": str(self.android_ndk_root),
            f"CC_{target_upper}": str(toolchain_dir / "bin" / f"{clang_target}-clang"),
            f"CXX_{target_upper}": str(toolchain_dir / "bin" / f"{clang_target}-clang++"),
            f"AR_{target_upper}": str(toolchain_dir / "bin" / "llvm-ar"),
            f"CARGO_TARGET_{target_upper}_LINKER": str(toolchain_dir / "bin" / f"{clang_target}-clang"),
        })
        
        return env
    
    def _build_target(self, target: Target) -> bool:
        """Build a specific target"""
        logger.info(f"Building target: {target.name} ({target.rust_target})")
        
        # Clean previous build
        target_dir = self.build_dir / target.rust_target
        if target_dir.exists():
            shutil.rmtree(target_dir)
        
        # Set up environment
        env = os.environ.copy()
        if target.platform == Platform.ANDROID:
            if not self.android_ndk_root:
                logger.error(f"Skipping Android target {target.name} - NDK not found")
                return False
            env = self._setup_android_environment(target)
        
        # Build command
        cmd = ["cargo", "build", "--target", target.rust_target]
        if self.mode == BuildMode.RELEASE:
            cmd.append("--release")
        
        # Execute build
        success = self._run_command(cmd, env=env)
        
        if success:
            # Verify the library was created
            mode_dir = self.mode.value
            lib_path = self.build_dir / target.rust_target / mode_dir / self._get_library_name(target)
            
            if lib_path.exists():
                logger.info(f"Successfully built {target.name}")
                return True
            else:
                logger.error(f"Library not found: {lib_path}")
                return False
        
        return False
    
    def _get_library_name(self, target: Target) -> str:
        """Get the expected library filename for a target"""
        if target.platform == Platform.WINDOWS:
            return "rvpnse.dll"
        elif target.platform in (Platform.MACOS, Platform.IOS):
            return "librvpnse.dylib"
        else:
            return "librvpnse.so"
    
    def _package_target(self, target: Target) -> Optional[Path]:
        """Package a built target into a distributable archive"""
        logger.info(f"Packaging target: {target.name}")
        
        mode_dir = self.mode.value
        lib_path = self.build_dir / target.rust_target / mode_dir / self._get_library_name(target)
        
        if not lib_path.exists():
            logger.error(f"Library not found for packaging: {lib_path}")
            return None
        
        # Create package directory
        package_dir = self.output_dir / f"rvpnse-{target.name}"
        package_dir.mkdir(parents=True, exist_ok=True)
        
        # Copy library
        shutil.copy2(lib_path, package_dir)
        
        # Copy headers and docs
        header_file = self.project_root / "include" / "rvpnse.h"
        if header_file.exists():
            shutil.copy2(header_file, package_dir)
        
        readme_file = self.project_root / "README.md"
        if readme_file.exists():
            shutil.copy2(readme_file, package_dir)
        
        # Create archive
        if target.platform == Platform.WINDOWS:
            archive_path = self.output_dir / f"rvpnse-{target.name}.zip"
            shutil.make_archive(str(archive_path.with_suffix("")), 'zip', package_dir)
        else:
            archive_path = self.output_dir / f"rvpnse-{target.name}.tar.gz"
            shutil.make_archive(str(archive_path.with_suffix("").with_suffix("")), 'gztar', package_dir)
        
        # Clean up temporary directory
        shutil.rmtree(package_dir)
        
        logger.info(f"Created package: {archive_path}")
        return archive_path
    
    def _package_android_bundle(self, android_targets: List[Target]) -> Optional[Path]:
        """Package all Android libraries into a single bundle"""
        logger.info("Creating Android bundle...")
        
        # Create bundle structure
        bundle_dir = self.output_dir / "rvpnse-android"
        jni_libs_dir = bundle_dir / "jniLibs"
        jni_libs_dir.mkdir(parents=True, exist_ok=True)
        
        mode_dir = self.mode.value
        
        # Copy libraries for each architecture
        for target in android_targets:
            lib_path = self.build_dir / target.rust_target / mode_dir / "librvpnse.so"
            if lib_path.exists():
                arch_dir = jni_libs_dir / target.android_arch
                arch_dir.mkdir(exist_ok=True)
                shutil.copy2(lib_path, arch_dir / "librvpnse.so")
                logger.info(f"Added {target.android_arch} library to bundle")
            else:
                logger.warning(f"Missing library for {target.name}")
        
        # Add headers and documentation
        header_file = self.project_root / "include" / "rvpnse.h"
        if header_file.exists():
            shutil.copy2(header_file, bundle_dir)
        
        readme_file = self.project_root / "README.md"
        if readme_file.exists():
            shutil.copy2(readme_file, bundle_dir)
        
        # Create installation script
        install_script = bundle_dir / "install.sh"
        install_script.write_text("""#!/bin/bash
# rVPNSE Android Library Installer

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <flutter_project_path>"
    echo "Example: $0 /path/to/WorxVPN"
    exit 1
fi

FLUTTER_PROJECT="$1"
JNI_LIBS_DIR="$FLUTTER_PROJECT/android/app/src/main/jniLibs"

if [ ! -d "$FLUTTER_PROJECT" ]; then
    echo "Error: Flutter project not found: $FLUTTER_PROJECT"
    exit 1
fi

echo "Installing rVPNSE Android libraries to $JNI_LIBS_DIR"

# Remove old libraries
rm -rf "$JNI_LIBS_DIR"
mkdir -p "$JNI_LIBS_DIR"

# Copy new libraries
cp -r jniLibs/* "$JNI_LIBS_DIR/"

echo "Installation complete!"
echo "Libraries installed:"
find "$JNI_LIBS_DIR" -name "*.so" -type f
""")
        install_script.chmod(0o755)
        
        # Create archive
        archive_path = self.output_dir / "rvpnse-android.tar.gz"
        shutil.make_archive(str(archive_path.with_suffix("").with_suffix("")), 'gztar', bundle_dir)
        
        # Clean up temporary directory
        shutil.rmtree(bundle_dir)
        
        logger.info(f"Created Android bundle: {archive_path}")
        return archive_path
    
    def _package_ios_bundle(self, ios_targets: List[Target]) -> Optional[Path]:
        """Package all iOS libraries into separate framework bundles"""
        logger.info("Creating iOS bundle...")
        
        # Create bundle structure
        bundle_dir = self.output_dir / "rvpnse-ios"
        bundle_dir.mkdir(parents=True, exist_ok=True)
        
        mode_dir = self.mode.value
        
        # Group targets by device/simulator
        device_targets = []
        simulator_targets = []
        
        for target in ios_targets:
            lib_path = self.build_dir / target.rust_target / mode_dir / "librvpnse.dylib"
            if lib_path.exists():
                if target.rust_target == "aarch64-apple-ios":
                    device_targets.append((target, lib_path))
                else:  # simulator targets
                    simulator_targets.append((target, lib_path))
                logger.info(f"Found library for {target.name}")
            else:
                logger.warning(f"Missing library for {target.name}: {lib_path}")
        
        if not device_targets and not simulator_targets:
            logger.error("No iOS libraries found to bundle")
            return None
        
        # Create device framework if we have device targets
        if device_targets:
            device_framework_dir = bundle_dir / "rVPNSE-Device.framework"
            device_framework_dir.mkdir(parents=True, exist_ok=True)
            
            if len(device_targets) == 1:
                shutil.copy2(device_targets[0][1], device_framework_dir / "rVPNSE")
            else:
                # Multiple device architectures, use lipo to combine
                device_paths = [str(path) for _, path in device_targets]
                lipo_cmd = ["lipo", "-create", "-output", str(device_framework_dir / "rVPNSE")] + device_paths
                if not self._run_command(lipo_cmd):
                    logger.error("Failed to create device framework")
                    return None
        
        # Create simulator framework if we have simulator targets
        if simulator_targets:
            sim_framework_dir = bundle_dir / "rVPNSE-Simulator.framework"
            sim_framework_dir.mkdir(parents=True, exist_ok=True)
            
            if len(simulator_targets) == 1:
                shutil.copy2(simulator_targets[0][1], sim_framework_dir / "rVPNSE")
            else:
                # Multiple simulator architectures, use lipo to combine
                sim_paths = [str(path) for _, path in simulator_targets]
                lipo_cmd = ["lipo", "-create", "-output", str(sim_framework_dir / "rVPNSE")] + sim_paths
                if not self._run_command(lipo_cmd):
                    logger.error("Failed to create simulator framework")
                    return None
        
        # Create Info.plist and Headers for both frameworks
        info_plist_content = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>rVPNSE</string>
    <key>CFBundleIdentifier</key>
    <string>com.devstroop.rvpnse</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>rVPNSE</string>
    <key>CFBundlePackageType</key>
    <string>FMWK</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>MinimumOSVersion</key>
    <string>11.0</string>
</dict>
</plist>"""
        
        module_map_content = """framework module rVPNSE {
    header "rvpnse.h"
    export *
}"""
        
        for framework_name in ["rVPNSE-Device.framework", "rVPNSE-Simulator.framework"]:
            framework_path = bundle_dir / framework_name
            if framework_path.exists():
                # Create Info.plist
                info_plist = framework_path / "Info.plist"
                info_plist.write_text(info_plist_content)
                
                # Create Headers directory and copy header
                headers_dir = framework_path / "Headers"
                headers_dir.mkdir(exist_ok=True)
                
                header_file = self.project_root / "include" / "rvpnse.h"
                if header_file.exists():
                    shutil.copy2(header_file, headers_dir)
                    
                    # Create module.modulemap
                    module_map = headers_dir / "module.modulemap"
                    module_map.write_text(module_map_content)
        
        # Add documentation
        readme_file = self.project_root / "README.md"
        if readme_file.exists():
            shutil.copy2(readme_file, bundle_dir)
        
        # Create installation script
        install_script = bundle_dir / "install.sh"
        install_script.write_text("""#!/bin/bash
# rVPNSE iOS Framework Installer

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <ios_project_path>"
    echo "Example: $0 /path/to/WorxVPN/ios"
    exit 1
fi

IOS_PROJECT="$1"
DEVICE_FRAMEWORK="rVPNSE-Device.framework"
SIMULATOR_FRAMEWORK="rVPNSE-Simulator.framework"

if [ ! -d "$IOS_PROJECT" ]; then
    echo "Error: iOS project not found: $IOS_PROJECT"
    exit 1
fi

# Find the Runner.xcodeproj
XCODEPROJ=$(find "$IOS_PROJECT" -name "*.xcodeproj" | head -n 1)
if [ -z "$XCODEPROJ" ]; then
    echo "Error: No Xcode project found in $IOS_PROJECT"
    exit 1
fi

FRAMEWORKS_DIR="$IOS_PROJECT/Frameworks"
mkdir -p "$FRAMEWORKS_DIR"

echo "Installing rVPNSE frameworks to $FRAMEWORKS_DIR"

# Remove old frameworks
rm -rf "$FRAMEWORKS_DIR/$DEVICE_FRAMEWORK"
rm -rf "$FRAMEWORKS_DIR/$SIMULATOR_FRAMEWORK"

# Copy new frameworks
if [ -d "$DEVICE_FRAMEWORK" ]; then
    cp -r "$DEVICE_FRAMEWORK" "$FRAMEWORKS_DIR/"
    echo "Device framework installed"
fi

if [ -d "$SIMULATOR_FRAMEWORK" ]; then
    cp -r "$SIMULATOR_FRAMEWORK" "$FRAMEWORKS_DIR/"
    echo "Simulator framework installed"
fi

echo "Installation complete!"
echo ""
echo "Manual steps required:"
echo "1. Open your iOS project in Xcode"
echo "2. Add the appropriate framework to your project:"
echo "   - Use rVPNSE-Device.framework for device builds"
echo "   - Use rVPNSE-Simulator.framework for simulator builds"
echo "3. Embed the framework in your target"
echo "4. Consider creating build phases to automatically select the correct framework"
""")
        install_script.chmod(0o755)
        
        # Create archive
        archive_path = self.output_dir / "rvpnse-ios.tar.gz"
        shutil.make_archive(str(archive_path.with_suffix("").with_suffix("")), 'gztar', bundle_dir)
        
        # Clean up temporary directory
        shutil.rmtree(bundle_dir)
        
        logger.info(f"Created iOS bundle: {archive_path}")
        return archive_path

    def build(self, target_names: List[str]) -> Dict[str, bool]:
        """Build specified targets"""
        logger.info(f"Building rVPNSE in {self.mode.value} mode")
        
        # Set up environment
        if not self._setup_rust_environment():
            return {}
        
        # Clean output directory
        if self.output_dir.exists():
            shutil.rmtree(self.output_dir)
        self.output_dir.mkdir(parents=True)
        
        # Validate targets
        invalid_targets = [name for name in target_names if name not in TARGETS]
        if invalid_targets:
            logger.error(f"Invalid targets: {invalid_targets}")
            logger.info(f"Available targets: {list(TARGETS.keys())}")
            return {}
        
        # Build targets
        results = {}
        android_targets = []
        ios_targets = []
        
        for target_name in target_names:
            target = TARGETS[target_name]
            success = self._build_target(target)
            results[target_name] = success
            
            if success:
                if target.platform == Platform.ANDROID:
                    android_targets.append(target)
                elif target.platform == Platform.IOS:
                    ios_targets.append(target)
                else:
                    # Package individual desktop targets
                    self._package_target(target)
        
        # Create Android bundle if we have Android targets
        if android_targets:
            self._package_android_bundle(android_targets)
        
        # Create iOS bundle if we have iOS targets
        if ios_targets:
            self._package_ios_bundle(ios_targets)
        
        return results
    
    def clean(self):
        """Clean build artifacts"""
        logger.info("Cleaning build artifacts...")
        
        if self.build_dir.exists():
            shutil.rmtree(self.build_dir)
            logger.info("Cleaned target directory")
        
        if self.output_dir.exists():
            shutil.rmtree(self.output_dir)
            logger.info("Cleaned output directory")
    
    def list_targets(self):
        """List available build targets"""
        print("\nAvailable build targets:")
        print("=" * 50)
        
        by_platform = {}
        for name, target in TARGETS.items():
            platform_name = target.platform.value.title()
            if platform_name not in by_platform:
                by_platform[platform_name] = []
            by_platform[platform_name].append((name, target))
        
        for platform, targets in by_platform.items():
            print(f"\n{platform}:")
            for name, target in targets:
                print(f"  {name:15} - {target.rust_target}")
                if target.android_arch:
                    print(f"                  Android arch: {target.android_arch}")

def main():
    parser = argparse.ArgumentParser(description="rVPNSE Unified Build System")
    parser.add_argument("--mode", choices=["debug", "release"], default="release",
                       help="Build mode (default: release)")
    parser.add_argument("--targets", nargs="+", 
                       help="Targets to build (use --list to see available)")
    parser.add_argument("--list", action="store_true",
                       help="List available targets")
    parser.add_argument("--clean", action="store_true",
                       help="Clean build artifacts")
    parser.add_argument("--all-desktop", action="store_true",
                       help="Build all desktop targets")
    parser.add_argument("--all-android", action="store_true",
                       help="Build all Android targets")
    parser.add_argument("--all-ios", action="store_true",
                       help="Build all iOS targets")
    parser.add_argument("--all", action="store_true",
                       help="Build all targets")
    parser.add_argument("--verbose", action="store_true",
                       help="Enable verbose logging")
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    # Find project root (parent of tools directory)
    project_root = Path(__file__).parent.parent
    builder = rVPNSEBuilder(project_root, BuildMode(args.mode))
    
    if args.list:
        builder.list_targets()
        return
    
    if args.clean:
        builder.clean()
        return
    
    # Determine targets to build
    targets_to_build = []
    
    if args.all:
        targets_to_build = list(TARGETS.keys())
    elif args.all_desktop:
        targets_to_build = [name for name, target in TARGETS.items() 
                           if target.platform not in (Platform.ANDROID, Platform.IOS)]
    elif args.all_android:
        targets_to_build = [name for name, target in TARGETS.items() 
                           if target.platform == Platform.ANDROID]
    elif args.all_ios:
        targets_to_build = [name for name, target in TARGETS.items() 
                           if target.platform == Platform.IOS]
    elif args.targets:
        targets_to_build = args.targets
    else:
        # Default to current platform
        current_system = platform.system().lower()
        if current_system == "darwin":
            if platform.machine() == "arm64":
                targets_to_build = ["macos-arm64"]
            else:
                targets_to_build = ["macos-x64"]
        elif current_system == "linux":
            targets_to_build = ["linux-x64"]
        elif current_system == "windows":
            targets_to_build = ["windows-x64"]
    
    if not targets_to_build:
        logger.error("No targets specified. Use --list to see available targets.")
        return 1
    
    # Build
    logger.info(f"Building targets: {targets_to_build}")
    results = builder.build(targets_to_build)
    
    # Report results
    print("\n" + "=" * 50)
    print("BUILD SUMMARY")
    print("=" * 50)
    
    success_count = 0
    for target, success in results.items():
        status = "SUCCESS" if success else "FAILED"
        print(f"{target:15} - {status}")
        if success:
            success_count += 1
    
    print(f"\nBuilt {success_count}/{len(results)} targets successfully")
    
    if success_count > 0:
        print(f"\nOutput files in: {builder.output_dir}")
    
    return 0 if success_count == len(results) else 1

if __name__ == "__main__":
    sys.exit(main())
