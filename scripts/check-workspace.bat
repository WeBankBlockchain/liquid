@echo off
setlocal enabledelayedexpansion

rem Please execute this script from the root of the project's directory.

set all_crates=(abi-codec macro primitives alloc core lang ty_mapping)
set results=()

set results[0].name=check_all_features
set results[0].result=0
for %%c in %all_crates% do (
    cargo +nightly check --verbose --manifest-path %%c\Cargo.toml
    if !errorlevel! neq 0 (
        set results[0].result=1
    )

    cargo +nightly check --verbose --no-default-features --features "contract" --manifest-path %%c\Cargo.toml --target=wasm32-unknown-unknown
    if !errorlevel! neq 0 (
        set results[0].result=1
    )
)

set results[1].name=build_wasm
set results[1].result=0
for %%c in %all_crates% do (
    cargo +nightly build --verbose --manifest-path %%c\Cargo.toml --no-default-features --release --target=wasm32-unknown-unknown
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

set results[3].name=clippy_all_features
set results[3].result=0
cargo +nightly clippy --verbose --all --all-features -- -D warnings
if !errorlevel! neq 0 (
    set results[3].result=1
)

set results[4].name=test_all_features
set results[4].result=0
cargo +nightly test --verbose --all --all-features --release
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
