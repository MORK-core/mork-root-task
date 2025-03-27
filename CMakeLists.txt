cmake_minimum_required(VERSION 3.10)
project(mork-root-task)

if(CMAKE_SOURCE_DIR STREQUAL PROJECT_SOURCE_DIR)
    message("Root Task use local config")
    include(${PROJECT_SOURCE_DIR}/config.cmake)
endif()

set(ROOT_TASK_DIR ${CMAKE_CURRENT_SOURCE_DIR})
set(ROOT_TASK_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/root-task")

set(LINKER_SCRIPT "${CMAKE_CURRENT_SOURCE_DIR}/linker-qemu-user.ld")

set(RUSTFLAGS "-C link-arg=-T${LINKER_SCRIPT} -C relocation-model=static")

set(ROOT_TASK_FILE ${ROOT_TASK_TARGET_DIR}/${TARGET}/${CARGO_BUILD_TYPE}/mork-root-task)

add_custom_target(root-task-build ALL
    COMMAND ${CMAKE_COMMAND} -E
    env "RUSTFLAGS=${RUSTFLAGS}" "CARGO_TARGET_DIR=${ROOT_TASK_TARGET_DIR}"
    ${CARGO_COMMAND} build ${CARGO_ARGS}
    WORKING_DIRECTORY ${ROOT_TASK_DIR}
    COMMENT "Building root task, ${ROOT_TASK_FILE}"
)

add_custom_command(
    OUTPUT ${CMAKE_BINARY_DIR}/${ROOT_TASK_FILE}.bin
    COMMAND ${OBJCOPY} --strip-all ${ROOT_TASK_FILE} ${ROOT_TASK_FILE}.bin
    COMMAND ${CMAKE_COMMAND} -E copy ${ROOT_TASK_FILE}.bin ${CMAKE_BINARY_DIR}
    DEPENDS root-task-build
    COMMENT "Stripping symbols from root_task"
)

add_custom_target(root_task_target ALL
  DEPENDS ${CMAKE_BINARY_DIR}/${ROOT_TASK_FILE}.bin
)
