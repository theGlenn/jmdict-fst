cmake_minimum_required(VERSION 3.10)

set(PROJECT_NAME "jmdict_fast")
project(${PROJECT_NAME} LANGUAGES CXX)

# Cargokit drives the Rust cross-compile. Args: target name, path to the
# cargo crate (vendored under `flutter_package/rust/`), cargo lib name.
include("../cargokit/cmake/cargokit.cmake")
apply_cargokit(${PROJECT_NAME} ../rust/ jmdict_fast_flutter "")

# Bundle the cargo-produced shared lib alongside the plugin.
set(jmdict_fast_bundled_libraries
  "${${PROJECT_NAME}_cargokit_lib}"
  PARENT_SCOPE
)
