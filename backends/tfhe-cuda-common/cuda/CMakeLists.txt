cmake_minimum_required(VERSION 3.18)
project(
  CommonCudaBackend
  VERSION 1.0.0
  LANGUAGES CXX)

set(MINIMUM_SUPPORTED_CUDA_VERSION 10.0)
include(CheckLanguage)
check_language(CUDA)
if(CMAKE_CUDA_COMPILER)
  enable_language(CUDA)
else()
  message(FATAL_ERROR "CUDA compiler not found.")
endif()

if(CMAKE_CUDA_COMPILER_VERSION VERSION_LESS ${MINIMUM_SUPPORTED_CUDA_VERSION})
  message(FATAL_ERROR "CUDA ${MINIMUM_SUPPORTED_CUDA_VERSION} or greater is required for compilation.")
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CUDA_STANDARD 17)
set(CMAKE_CUDA_STANDARD_REQUIRED ON)

# Auto-detect CUDA compute capability
set(CUDAFILE ${CMAKE_CURRENT_SOURCE_DIR}/check_cuda.cu)
if(WIN32)
  set(OUTPUTFILE ${CMAKE_CURRENT_SOURCE_DIR}/cuda_script.exe)
  execute_process(
    COMMAND ${CMAKE_CXX_COMPILER} /TP /FIcuda_runtime.h /I"$ENV{CUDA_PATH}/include" ${CUDAFILE} /link /LIBPATH:"$ENV{CUDA_PATH}/lib/x64" cudart.lib /OUT:${OUTPUTFILE}
    RESULT_VARIABLE COMPILE_RESULT
    OUTPUT_VARIABLE COMPILE_OUTPUT
    ERROR_VARIABLE COMPILE_ERROR
  )
  if(NOT COMPILE_RESULT EQUAL 0)
    message(WARNING "Failed to compile check_cuda: ${COMPILE_ERROR}")
  endif()
  # Also remove the generated object file check_cuda.obj
  file(REMOVE ${CMAKE_CURRENT_SOURCE_DIR}/check_cuda.obj)
else()
  set(OUTPUTFILE ${CMAKE_CURRENT_SOURCE_DIR}/cuda_script)
  execute_process(COMMAND nvcc -lcuda ${CUDAFILE} -o ${OUTPUTFILE})
endif()

execute_process(
  COMMAND ${OUTPUTFILE}
  RESULT_VARIABLE CUDA_RETURN_CODE
  OUTPUT_VARIABLE ARCH)
file(REMOVE ${OUTPUTFILE})

if(${CUDA_RETURN_CODE} EQUAL 0)
  set(CUDA_SUCCESS "TRUE")
else()
  set(CUDA_SUCCESS "FALSE")
endif()

if(${CUDA_SUCCESS})
  set(CMAKE_CUDA_ARCHITECTURES native)
  string(REPLACE "-arch=sm_" "" CUDA_ARCH "${ARCH}")
  set(CUDA_ARCH "${CUDA_ARCH}0")
else()
  set(CMAKE_CUDA_ARCHITECTURES 70)
  set(CUDA_ARCH "700")
endif()

add_compile_definitions(CUDA_ARCH=${CUDA_ARCH})

set(CMAKE_CUDA_SEPARABLE_COMPILATION ON)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE
      Release
      CACHE STRING "Build type" FORCE)
endif()

if(NOT MSVC)
  set(CMAKE_CXX_FLAGS_RELEASE "-O3 -DNDEBUG")
  set(CMAKE_CUDA_FLAGS_RELEASE "-O3 -DNDEBUG")
  set(CMAKE_CXX_FLAGS_DEBUG "-g -O0")
  set(CMAKE_CUDA_FLAGS_DEBUG "-g -O0 -G")
else()
  set(CMAKE_CXX_FLAGS_RELEASE "/O2 /DNDEBUG")
  set(CMAKE_CUDA_FLAGS_RELEASE "-O3 -DNDEBUG")
  set(CMAKE_CXX_FLAGS_DEBUG "/Od /Zi")
  set(CMAKE_CUDA_FLAGS_DEBUG "-g -O0 -G")
endif()

if(${USE_NVTOOLS})
  add_definitions(-DUSE_NVTOOLS)
endif()

if(NOT MSVC)
  set(CMAKE_CUDA_FLAGS "${CMAKE_CUDA_FLAGS} -Xcompiler -Wall -Xcompiler -Wextra")
else()
  set(CMAKE_CUDA_FLAGS "${CMAKE_CUDA_FLAGS} -Xcompiler /W3")
endif()
set(CMAKE_CUDA_FLAGS "${CMAKE_CUDA_FLAGS} --use_fast_math --expt-relaxed-constexpr")

set(COMMON_SOURCES src/device.cu src/helper_profile.cu)

add_library(tfhe_cuda_common STATIC ${COMMON_SOURCES})

set_target_properties(
  tfhe_cuda_common
  PROPERTIES CUDA_SEPARABLE_COMPILATION ON
             POSITION_INDEPENDENT_CODE ON
             CUDA_RESOLVE_DEVICE_SYMBOLS ON)

if(CMAKE_BUILD_TYPE STREQUAL "Release")
  set_target_properties(tfhe_cuda_common PROPERTIES CUDA_OPTIMIZE_DEPENDENCIES ON)
elseif(CMAKE_BUILD_TYPE STREQUAL "Debug")
  target_compile_options(tfhe_cuda_common PRIVATE $<$<COMPILE_LANGUAGE:CUDA>:-g -O0 -G>)
endif()

target_include_directories(tfhe_cuda_common PUBLIC include)
target_link_libraries(tfhe_cuda_common PUBLIC cudart)

install(
  TARGETS tfhe_cuda_common
  ARCHIVE DESTINATION lib
  LIBRARY DESTINATION lib)
