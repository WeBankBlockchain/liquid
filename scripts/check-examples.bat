@echo off
setlocal enabledelayedexpansion

rem Please execute this script from the root of the project's directory.

set all_examples=(hello_world)
set results=()

set results[0].name=wasm_test
set results[0].result=0

set results[1].name=unit_test
set results[1].result=0

set results[2].name=abi_test
set results[2].result=0

for /f %%f in ('dir /b examples') do (
    echo %%f

    cargo +nightly build --release --no-default-features --target=wasm32-unknown-unknown --verbose --manifest-path examples/%%f/Cargo.toml
    if !errorlevel! neq 0 (
        set results[0].result=1
    )

    cargo +nightly test --verbose --manifest-path examples/%%f/Cargo.toml
    if !errorlevel! neq 0 (
        set results[1].result=1
    )

    cargo +nightly run --package abi-gen --manifest-path examples/%%f/Cargo.toml
    if !errorlevel! neq 0 (
        set results[2].result=1
    )
)

set all_check_passed=0
set banner=-----------------

echo Examples Results
echo %banner%

for /l %%i in (0,1,2) do (
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
