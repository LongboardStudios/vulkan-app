# Vulkan App

This is a cross-platform windowed application to use as a base for Vulkan development.

Works on Linux, Mac and Windows.

Developed in Rust.

## Prerequisites
Rust and Cargo stable (use [Rustup](https://rustup.rs) to install).

## Getting Started

### Mac

Vulkan support on macOS is provided by the MoltenVK runtime library.

If this is not present, the app will panic with error:
`LoadingError(LibraryLoadFailure("dlopen(libvulkan.1.dylib, 1): image not found"`

1. Download the [Vulkan SDK from LunarG](https://vulkan.lunarg.com/sdk/home#mac)
2. Install with `python install_vulkan.py` in the downloaded directory. This places the required libraries and binaries in `/usr/local/`

### Linux (work in progress)
1. Make sure you have Vulkan drivers installed.
2. Install deps (vulkan sdk, cmake, etc.)

### Windows (currently untested)
TODO

## Running
`cargo run`

## Notes
The project uses wininit for window and events.

Vulkano is used for a pleasant rust Vulkan experience, at the cost of ultra-fine-grained tweaking ability. For example, [this method of debugging instance creation and destruction](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Validation_layers#page_Debugging-instance-creation-and-destruction) is not currently possible with Vulkano.

A CLion project configuration is included, and this is the recommended development setup on all platforms.