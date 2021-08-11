@echo off
setlocal enabledelayedexpansion

rem Please execute this script from the root of the project's directory.

set CARGO_TARGET_DIR=.\examples\target

for /f %%d in ('dir /b examples') do (
    if /i %%d neq target (
        for /f %%f in ('dir /b examples\%%d') do (
            cargo build --release --no-default-features --target=wasm32-unknown-unknown --manifest-path examples\%%d\%%f\Cargo.toml
            if !errorlevel! neq 0 (
                exit /b %errorlevel%
            )

            cargo test --manifest-path examples\%%d\%%f\Cargo.toml
            if !errorlevel! neq 0 (
                exit /b %errorlevel%
            )

            cargo run --package abi-gen --manifest-path examples\%%d\%%f\Cargo.toml
            if !errorlevel! neq 0 (
                exit /b %errorlevel%
            )

            cargo build --release --no-default-features --features "gm" --target=wasm32-unknown-unknown --manifest-path examples\%%d\%%f\Cargo.toml
            if !errorlevel! neq 0 (
                exit /b %errorlevel%
            )
        )
    )
)
