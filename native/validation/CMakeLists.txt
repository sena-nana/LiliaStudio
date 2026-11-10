cmake_minimum_required(VERSION 3.20)
project(ameya_native_layout_validation LANGUAGES CXX)

include(CTest)

add_executable(layout_snapshot_smoke layout_snapshot_smoke.cpp)
target_compile_features(layout_snapshot_smoke PRIVATE cxx_std_17)

add_executable(content_snapshot_acceptance content_snapshot_acceptance.cpp)
target_compile_features(content_snapshot_acceptance PRIVATE cxx_std_17)

if(BUILD_TESTING)
  add_test(
    NAME native.layout_snapshot_smoke
    COMMAND layout_snapshot_smoke
            ${CMAKE_CURRENT_SOURCE_DIR}/fixtures/minimal.snapshot
  )
  set_tests_properties(native.layout_snapshot_smoke PROPERTIES
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
  )

  add_test(
    NAME native.layout_snapshot_rejects_overlap
    COMMAND layout_snapshot_smoke
            ${CMAKE_CURRENT_SOURCE_DIR}/fixtures/invalid-overlap.snapshot
  )
  set_tests_properties(native.layout_snapshot_rejects_overlap PROPERTIES
    WILL_FAIL TRUE
  )

  add_test(
    NAME native.layout_snapshot_rejects_invalid_margin
    COMMAND layout_snapshot_smoke
            ${CMAKE_CURRENT_SOURCE_DIR}/fixtures/invalid-margin.snapshot
  )
  set_tests_properties(native.layout_snapshot_rejects_invalid_margin PROPERTIES
    WILL_FAIL TRUE
  )

  add_test(
    NAME native.offline_content_acceptance
    COMMAND content_snapshot_acceptance
            --contract ${CMAKE_CURRENT_SOURCE_DIR}/../../artifacts/native-snapshots/content-acceptance.tsv
            --root ${CMAKE_CURRENT_SOURCE_DIR}/../../artifacts/native-snapshots
  )
  set_tests_properties(native.offline_content_acceptance PROPERTIES
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
  )

  file(GLOB NATIVE_OFFLINE_SNAPSHOTS
       "${CMAKE_CURRENT_SOURCE_DIR}/../../artifacts/native-snapshots/*/*.snapshot")
  foreach(snapshot IN LISTS NATIVE_OFFLINE_SNAPSHOTS)
    get_filename_component(snapshot_name "${snapshot}" NAME_WE)
    get_filename_component(snapshot_state "${snapshot}" DIRECTORY)
    get_filename_component(snapshot_state "${snapshot_state}" NAME)
    add_test(
      NAME "native.offline_${snapshot_state}_${snapshot_name}"
      COMMAND layout_snapshot_smoke "${snapshot}"
    )
    set_tests_properties("native.offline_${snapshot_state}_${snapshot_name}"
      PROPERTIES WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})
  endforeach()
endif()
