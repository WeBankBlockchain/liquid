@echo off
setlocal enabledelayedexpansion

rem Please execute this script from the root of the project's directory.

set features=("contract,solidity-compatible", "collaboration")
set results=()

set results[0].name=basic_check
set results[0].result=0
for %%f in %features% do (
    cargo +nightly check --verbose --features %%f --manifest-path lang\Cargo.toml
    if !errorlevel! neq 0 (
        set results[0].result=1
    )

    cargo +nightly check --verbose --no-default-features --features %%f --target=wasm32-unknown-unknown --manifest-path lang\Cargo.toml
    if !errorlevel! neq 0 (
        set results[0].result=1
    )
)

set results[1].name=build_wasm
set results[1].result=0
for %%f in %features% do (
    cargo +nightly build --verbose --no-default-features --features %%f --release --target=wasm32-unknown-unknown --manifest-path lang\Cargo.toml
    if !errorlevel! neq 0 (
        set results[1].result=1
    )
)

set results[2].name=fmt
set results[2].result=0
cargo +nightly fmt --verbose --all -- --check
if !errorlevel! neq 0 (
    set result[2].result=1
)

set results[3].name=clippy
set results[3].result=0
for %%f in %features% do (
    cargo +nightly clippy --verbose --all --features %%f --manifest-path lang\Cargo.toml -- -D warnings 
    if !errorlevel! neq 0 (
        set results[3].result=1
    )

    cargo +nightly clippy --verbose --all --no-default-features --features %%f --target=wasm32-unknown-unknown --manifest-path lang\Cargo.toml -- -D warnings 
    if !errorlevel! neq 0 (
        set results[3].result=1
    )
)
if !errorlevel! neq 0 (
    set results[3].result=1
)

set results[4].name=unit_tests
set results[4].result=0
cargo +nightly test --verbose --features "contract,solidity-compatible" --release --manifest-path lang/Cargo.toml
cargo +nightly test --verbose --features "collaboration" --release --manifest-path lang/Cargo.toml
cargo +nightly test --verbose --release --manifest-path ty_mapping/Cargo.toml
cargo +nightly test --verbose --release --manifest-path primitives/Cargo.toml
cargo +nightly test --verbose --release --features "collaboration" --manifest-path lang/macro/Cargo.toml
if !errorlevel! neq 0 (
    set results[4].result=1
)

set all_check_passed=0
set banner=-----------------

echo Workspace Results
echo %banner%

for /l %%i in (0,1,4) do (
    set cur.name=
    set cur.result=

    for /f "usebackq delims=.= tokens=1-3" %%j in (`set results[%%i]`) do (
        set cur.%%k=%%l
    )

    set result_str=
    if !cur.result! equ 0 (
        set result_str=OK
    ) else (
        set result_str=ERROR
        set all_check_passed=1
    )
    echo - !cur.name! : !result_str!
)

echo.

if !all_check_passed! equ 0 (
    echo Workspace: All checks passed
    echo %banner%
) else (
    echo Workspace: Some checks failed
    echo %banner%
    exit /b 1
)
