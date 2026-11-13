#!/bin/sh
# Builds the gpui target's adapter (and with it the GPUI app, which it links) once. The adapter
# and the app share one build folder (.cargo/config.toml), so the GPUI crates compile once.
#
# Needs: Rust, and on Linux the X11, xkbcommon, fontconfig and Vulkan libraries GPUI links (the
# -dev packages, or PKG_CONFIG_PATH pointing at them), an X server or Xvfb, and a Vulkan driver
# (with no GPU, Mesa's lavapipe: VK_ICD_FILENAMES=<its lvp_icd.json>).
set -eu
cd "$(dirname "$0")"
cargo build --locked --bin taskscape-gpui-adapter
cargo build --locked --manifest-path ../../../gpui/Cargo.toml --bin taskscape-gpui
echo "gpui target: built ../../../gpui/target/debug/taskscape-gpui-adapter"
