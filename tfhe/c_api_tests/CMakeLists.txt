project(tfhe-c-api-tests)

cmake_minimum_required(VERSION 3.16)

if(NOT CARGO_PROFILE)
    set(CARGO_PROFILE release)
endif()
set(TFHE_C_API_RELEASE "${CMAKE_CURRENT_SOURCE_DIR}/../../target/${CARGO_PROFILE}")

option(WITH_FEATURE_GPU "Enable if tfhe-rs C API was compiled with the 'gpu' feature activated" OFF)

include_directories(${TFHE_C_API_RELEASE})
# This one is to fetch the dynamic buffer header
include_directories(${TFHE_C_API_RELEASE}/deps)
add_library(Tfhe STATIC IMPORTED)
if(MSVC)
    set_target_properties(Tfhe PROPERTIES IMPORTED_LOCATION ${TFHE_C_API_RELEASE}/tfhe.lib)
else()
    set_target_properties(Tfhe PROPERTIES IMPORTED_LOCATION ${TFHE_C_API_RELEASE}/libtfhe.a)
endif()

if(APPLE)
    find_library(SECURITY_FRAMEWORK Security)
    if (NOT SECURITY_FRAMEWORK)
        message(FATAL_ERROR "Security framework not found")
    endif()
endif()

if (WITH_FEATURE_GPU)
    find_package(CUDAToolkit 10.0 REQUIRED)
    find_package(OpenMP REQUIRED)
endif()

file(GLOB TEST_CASES test_*.c)
if(MSVC)
    list(REMOVE_ITEM TEST_CASES 
        "${CMAKE_CURRENT_SOURCE_DIR}/test_high_level_threading.c"
        "${CMAKE_CURRENT_SOURCE_DIR}/test_high_level_zk.c"
    )
endif()

foreach (testsourcefile ${TEST_CASES})
    get_filename_component(testname ${testsourcefile} NAME_WLE)
    get_filename_component(groupname ${testsourcefile} DIRECTORY)
    add_executable(${testname} ${testsourcefile})
    add_test(
            NAME ${testname}
            COMMAND ${testname}
            WORKING_DIRECTORY ${CMAKE_BINARY_DIR}/Testing
    )
    target_include_directories(${testname} PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
    
    if(MSVC)
        target_link_libraries(${testname} LINK_PUBLIC Tfhe ws2_32 userenv bcrypt advapi32 ntdll)
    else()
        target_link_libraries(${testname} LINK_PUBLIC Tfhe m pthread dl)
    endif()

    if (WITH_FEATURE_GPU)
        target_link_libraries(${testname} LINK_PUBLIC CUDA::cudart -lstdc++ OpenMP::OpenMP_CXX)
        target_compile_definitions(${testname} PUBLIC -DWITH_FEATURE_GPU)
    endif()

    if(APPLE)
        target_link_libraries(${testname} LINK_PUBLIC ${SECURITY_FRAMEWORK})
    endif()
    if(NOT MSVC)
        target_compile_options(${testname} PRIVATE -Werror)
    endif()
    # Enabled asserts even in release mode
    add_definitions(-UNDEBUG)
endforeach (testsourcefile ${TEST_CASES})
