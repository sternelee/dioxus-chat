#!/bin/bash

# Dioxus Chat Development Script

set -e

PLATFORM=$1
COMMAND=$2

function show_help() {
    echo "Usage: $0 <platform> <command>"
    echo ""
    echo "Platforms:"
    echo "  web      - Build and run web version"
    echo "  desktop  - Build and run desktop version"
    echo "  mobile   - Build and run mobile version"
    echo ""
    echo "Commands:"
    echo "  run      - Run the application"
    echo "  build    - Build the application"
    echo "  clean    - Clean build artifacts"
    echo ""
    echo "Examples:"
    echo "  $0 web run        # Run web version"
    echo "  $0 desktop build  # Build desktop version"
    echo "  $0 mobile run     # Run mobile version"
}

function run_web() {
    case $COMMAND in
        "run")
            echo "🚀 Starting web application..."
            cd packages/web
            cargo run
            ;;
        "build")
            echo "🔨 Building web application..."
            cd packages/web
            cargo build
            ;;
        "clean")
            echo "🧹 Cleaning web build artifacts..."
            cd packages/web
            cargo clean
            ;;
        *)
            echo "❌ Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
}

function run_desktop() {
    case $COMMAND in
        "run")
            echo "🚀 Starting desktop application..."
            cd packages/desktop
            cargo run
            ;;
        "build")
            echo "🔨 Building desktop application..."
            cd packages/desktop
            cargo build
            ;;
        "clean")
            echo "🧹 Cleaning desktop build artifacts..."
            cd packages/desktop
            cargo clean
            ;;
        *)
            echo "❌ Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
}

function run_mobile() {
    case $COMMAND in
        "run")
            echo "🚀 Starting mobile application..."
            cd packages/mobile
            cargo run
            ;;
        "build")
            echo "🔨 Building mobile application..."
            cd packages/mobile
            cargo build
            ;;
        "clean")
            echo "🧹 Cleaning mobile build artifacts..."
            cd packages/mobile
            cargo clean
            ;;
        *)
            echo "❌ Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
}

function run_all() {
    case $COMMAND in
        "run")
            echo "🚀 Starting all applications..."
            echo "Note: Running all platforms at once is not recommended"
            echo "Please run each platform separately"
            ;;
        "build")
            echo "🔨 Building all applications..."
            echo "Building web..."
            cd packages/web && cargo build
            cd ../..
            echo "Building desktop..."
            cd packages/desktop && cargo build
            cd ../..
            echo "Building mobile..."
            cd packages/mobile && cargo build
            cd ../..
            echo "✅ All applications built successfully!"
            ;;
        "clean")
            echo "🧹 Cleaning all build artifacts..."
            cd packages/web && cargo clean
            cd ../..
            cd packages/desktop && cargo clean
            cd ../..
            cd packages/mobile && cargo clean
            cd ../..
            echo "✅ All build artifacts cleaned!"
            ;;
        *)
            echo "❌ Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
}

# Main script logic
case $PLATFORM in
    "web")
        run_web
        ;;
    "desktop")
        run_desktop
        ;;
    "mobile")
        run_mobile
        ;;
    "all")
        run_all
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        echo "❌ Unknown platform: $PLATFORM"
        show_help
        exit 1
        ;;
esac