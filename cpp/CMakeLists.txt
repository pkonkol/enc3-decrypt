cmake_minimum_required(VERSION 3.10)

set(CMAKE_DISABLE_SOURCE_CHANGES ON)
set(CMAKE_DISABLE_IN_SOURCE_BUILD ON)

project(otcv8decrypt)

add_subdirectory(src)
add_executable(otcv8decrypt ${otcv8decrypt_SRC})

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")

set_target_properties(otcv8decrypt PROPERTIES CXX_STANDARD 17)
set_target_properties(otcv8decrypt PROPERTIES CXX_STANDARD_REQUIRED ON)

if (NOT WIN32)
    add_compile_options(-Wall -Werror -pipe -fvisibility=hidden)
endif ()

set(CMAKE_CXX_FLAGS_PERFORMANCE "${CMAKE_CXX_FLAGS_RELEASE} -march=native")

if (CMAKE_COMPILER_IS_GNUCXX)
    add_compile_options(-fno-strict-aliasing)
endif ()

find_package(ZLIB REQUIRED)

include_directories(${ZLIB_INCLUDE_DIR})
target_link_libraries(otcv8decrypt PRIVATE
        ${ZLIB_LIBRARY}
        )
