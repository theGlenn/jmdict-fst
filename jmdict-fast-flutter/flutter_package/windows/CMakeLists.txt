cmake_minimum_required(VERSION 3.14)

set(PROJECT_NAME "jmdict_fast")
project(${PROJECT_NAME} LANGUAGES CXX)

# Cargokit drives the Rust cross-compile. `manifest_dir` is relative to
# `CMAKE_CURRENT_SOURCE_DIR` (the plugin's `windows/`). The binding crate
# is vendored at `flutter_package/rust/` so it ships with the published
# pub.dev tarball.
include("../cargokit/cmake/cargokit.cmake")
apply_cargokit(${PROJECT_NAME} ../rust/ jmdict_fast_flutter "")

set(jmdict_fast_bundled_libraries
  "${${PROJECT_NAME}_cargokit_lib}"
  PARENT_SCOPE
)
