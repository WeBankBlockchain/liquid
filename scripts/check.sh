#!/bin/bash
# Please execute this script at the root of the project's directory.

set -e
check_examples_flag=
check_workspace_flag=
features=("contract,solidity-compatible" "collaboration")
build_log="./build.log"

LOG_WARN()
{
    local content=${1}
    echo -e "\033[31m[WARN] ${content}\033[0m"
}

LOG_INFO()
{
    local content=${1}
    echo -e "\033[32m[INFO] ${content}\033[0m"
}

help()
{
    cat << EOF
Please execute this script at the root of the project's directory.

Usage:
    -e <check examples>
    -w <check workspace>
    -h Help
e.g:
    bash $0 -e -w
EOF
exit 0
}

parse_params()
{
    echo "parse_params $#"
    while getopts "ewh" option;do
        case $option in
        e) check_examples_flag="true";;
        w) check_workspace_flag="true";;
        h) help;;
        *) LOG_WARN "invalid option $option";;
        esac
    done
    if [[ $# == 0 ]]; then
        check_examples_flag="true"
        check_workspace_flag="true"
    fi
}

check_examples() {
    for dir in $(ls examples);do
        for example in $(ls examples/${dir});do
            LOG_INFO "checking examples/${dir}/${example} ..."
            cargo +nightly build --release --no-default-features --target=wasm32-unknown-unknown --manifest-path "examples/${dir}/${example}/Cargo.toml"
            cargo +nightly test --manifest-path "examples/${dir}/${example}/Cargo.toml"
            cargo +nightly run --package abi-gen --manifest-path "examples/${dir}/${example}/Cargo.toml"
            cargo +nightly build --release --no-default-features --features "gm" --target=wasm32-unknown-unknown --manifest-path "examples/${dir}/${example}/Cargo.toml"
            LOG_INFO "examples/${dir}/${example} is ok."
        done
    done
}

check_workspace() {
    LOG_INFO "checking workspace build ..."
    for feature in ${features[*]};do
        LOG_INFO "checking feature ${feature} ..."
        cargo +nightly check --verbose --features "${feature}" --manifest-path lang/Cargo.toml
        cargo +nightly check --verbose --no-default-features --features "${feature}" --target=wasm32-unknown-unknown --manifest-path lang/Cargo.toml
        cargo +nightly build --verbose --no-default-features --features "${feature}" --release --target=wasm32-unknown-unknown --manifest-path lang/Cargo.toml > ${build_log}
    done
    LOG_INFO "checking workspace build fmt ..."
    cargo +nightly fmt --verbose --all -- --check
    LOG_INFO "checking workspace build unit test ..."
    cargo +nightly test --verbose --features "contract,solidity-compatible" --release --manifest-path lang/Cargo.toml
    cargo +nightly test --verbose --features "collaboration" --release --manifest-path lang/Cargo.toml
    cargo +nightly test --verbose --release --manifest-path ty_mapping/Cargo.toml
    cargo +nightly test --verbose --release --manifest-path primitives/Cargo.toml
    cargo +nightly test --verbose --release --features "collaboration" --manifest-path lang/macro/Cargo.toml
    for feature in ${features[*]};do
        LOG_INFO "checking feature ${feature} using clippy ..."
        cargo +nightly clippy --verbose --all --features "${feature}" --features "contract-abi-gen" --manifest-path lang/Cargo.toml -- -D warnings
        cargo +nightly clippy --verbose --all --no-default-features --features "${feature}" --features "collaboration-abi-gen" --target=wasm32-unknown-unknown --manifest-path lang/Cargo.toml -- -D warnings
    done
}

main(){
    if [[ "${check_examples_flag}" == "true" ]];then
        check_examples
    fi
    if [[ "${check_workspace_flag}" == "true" ]];then
        check_workspace
    fi
    if [ -f "${build_log}" ]; then
        rm  "${build_log}"
    fi
}

parse_params "$@"
main

