#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Default values
PUSH=false
PLATFORMS="linux/amd64,linux/arm64"
REGISTRY="ghcr.io/builds-toqyo/aether"
VERSION="latest"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --push)
            PUSH=true
            shift
            ;;
        --platforms)
            PLATFORMS="$2"
            shift 2
            ;;
        --registry)
            REGISTRY="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --push           Push images to registry"
            echo "  --platforms      Set platforms (default: linux/amd64,linux/arm64)"
            echo "  --registry       Set registry (default: ghcr.io/builds-toqyo/aether)"
            echo "  --version        Set version tag (default: latest)"
            echo "  --help           Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

print_status "Starting multi-platform Docker build for Aether"
print_status "Platforms: $PLATFORMS"
print_status "Registry: $REGISTRY"
print_status "Version: $VERSION"

# Check if Docker Buildx is installed
if ! docker buildx version > /dev/null 2>&1; then
    print_error "Docker Buildx is not installed. Please install it first."
    exit 1
fi

# Create and use buildx builder
BUILDER_NAME="aether-multiplatform"
if ! docker buildx inspect $BUILDER_NAME > /dev/null 2>&1; then
    print_status "Creating new buildx builder: $BUILDER_NAME"
    docker buildx create --name $BUILDER_NAME --use --bootstrap
else
    print_status "Using existing buildx builder: $BUILDER_NAME"
    docker buildx use $BUILDER_NAME
fi

# Build configurations
declare -A BUILD_CONFIGS=(
    ["ubuntu"]="ubuntu:jammy ubuntu:noble"
    ["debian"]="debian:bookworm debian:bullseye"
    ["alpine"]="alpine:latest"
)

# Build for each distribution
for distro in "${!BUILD_CONFIGS[@]}"; do
    print_status "Building for $distro family"
    
    IFS=' ' read -ra VERSIONS <<< "${BUILD_CONFIGS[$distro]}"
    for version in "${VERSIONS[@]}"; do
        print_status "Building $distro:$version"
        
        # Extract version number for tag
        version_tag=$(echo $version | cut -d':' -f2)
        tag="${REGISTRY}:${distro}-${version_tag}"
        
        if [ "$VERSION" != "latest" ]; then
            tag="${tag}-${VERSION}"
        fi
        
        # Build arguments
        build_args=""
        if [[ $distro == "ubuntu" ]]; then
            build_args="--build-arg UBUNTU_VERSION=$version_tag"
        elif [[ $distro == "debian" ]]; then
            build_args="--build-arg DEBIAN_VERSION=$version_tag"
        fi
        
        # Build image
        if docker buildx build \
            --platform $PLATFORMS \
            --tag $tag \
            $build_args \
            --file docker/${distro}.Dockerfile \
            --push=$PUSH \
            .; then
            print_status "Successfully built $tag"
        else
            print_error "Failed to build $tag"
            exit 1
        fi
    done
done

# Create multi-arch manifest for latest
if [ "$PUSH" = true ]; then
    print_status "Creating multi-arch manifest for latest"
    
    # Create manifest for each distribution
    for distro in "${!BUILD_CONFIGS[@]}"; do
        IFS=' ' read -ra VERSIONS <<< "${BUILD_CONFIGS[$distro]}"
        primary_version=$(echo ${VERSIONS[0]} | cut -d':' -f2)
        
        manifest_tag="${REGISTRY}:${distro}-latest"
        docker manifest create $manifest_tag
        
        # Add each platform image to manifest
        for platform in ${PLATFORMS//,/ }; do
            arch=$(echo $platform | cut -d'/' -f2)
            if [[ $arch == "amd64" ]]; then
                docker manifest add $manifest_tag "${REGISTRY}:${distro}-${primary_version}"
            else
                docker manifest add $manifest_tag "${REGISTRY}:${distro}-${primary_version}"
            fi
        done
        
        docker manifest push $manifest_tag
        print_status "Created manifest: $manifest_tag"
    done
fi

# Clean up builder
print_status "Cleaning up buildx builder"
docker buildx rm $BUILDER_NAME

print_status "Multi-platform Docker build completed successfully!"

# Display built images
if [ "$PUSH" = true ]; then
    print_status "Pushed images:"
    for distro in "${!BUILD_CONFIGS[@]}"; do
        IFS=' ' read -ra VERSIONS <<< "${BUILD_CONFIGS[$distro]}"
        for version in "${VERSIONS[@]}"; do
            version_tag=$(echo $version | cut -d':' -f2)
            tag="${REGISTRY}:${distro}-${version_tag}"
            if [ "$VERSION" != "latest" ]; then
                tag="${tag}-${VERSION}"
            fi
            echo "  - $tag"
        done
    done
fi
