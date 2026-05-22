cmake_minimum_required(VERSION 3.14)

set(PROJECT_NAME "jmdict_fast")
project(${PROJECT_NAME} LANGUAGES CXX)

# Cargokit drives the Rust cross-compile. Path is relative to the CMake
# build dir (not the source dir), which on Windows resolves under the
# consumer's build/windows/x64/plugins/<plugin>/ — hence the deep `../..`
# walk back to the binding crate.
include("../cargokit/cmake/cargokit.cmake")
apply_cargokit(${PROJECT_NAME} ../../../../../../ jmdict_fast_flutter "")

set(jmdict_fast_bundled_libraries
  "${${PROJECT_NAME}_cargokit_lib}"
  PARENT_SCOPE
)
