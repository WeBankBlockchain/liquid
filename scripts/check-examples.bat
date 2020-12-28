@echo off
setlocal enabledelayedexpansion

rem Please execute this script from the root of the project's directory.

for /f %%f in ('dir /b examples\contract') do (
    cargo +nightly build --release --no-default-features --target=wasm32-unknown-unknown --manifest-path examples\contract\%%f\Cargo.toml
    if !errorlevel! neq 0 (
        set results[0].result=1
    )

    cargo +nightly test --manifest-path examples\contract\%%f\Cargo.toml
    if !errorlevel! neq 0 (
        set results[1].result=1
    )

    cargo +nightly run --package abi-gen --manifest-path examples\contract\/%%f\Cargo.toml
    if !errorlevel! neq 0 (
        set results[2].result=1
    )

    cargo +nightly build --release --no-default-features --features "gm" --target=wasm32-unknown-unknown --manifest-path examples\contract\%%f\Cargo.toml
    if !errorlevel! neq 0 (
        set results[3].result=1
    )
)

for /f %%f in ('dir /b examples\collaboration') do (
    cargo +nightly build --release --no-default-features --target=wasm32-unknown-unknown --manifest-path examples\collaboration\%%f\Cargo.toml
    if !errorlevel! neq 0 (
        set results[0].result=1
    )

    cargo +nightly test --manifest-path examples\collaboration\%%f\Cargo.toml
    if !errorlevel! neq 0 (
        set results[1].result=1
    )

    cargo +nightly run --package abi-gen --manifest-path examples\collaboration\%%f\Cargo.toml
    if !errorlevel! neq 0 (
        set results[2].result=1
    )

    cargo +nightly build --release --no-default-features --features "gm" --target=wasm32-unknown-unknown --manifest-path examples\collaboration\%%f\Cargo.toml
    if !errorlevel! neq 0 (
        set results[3].result=1
    )
)

for /f %%f in ('dir /b examples\asset') do (
    cargo +nightly build --release --no-default-features --target=wasm32-unknown-unknown --manifest-path examples\asset\%%f\Cargo.toml
    if !errorlevel! neq 0 (
        set results[0].result=1
    )

    cargo +nightly test --manifest-path examples\asset\%%f\Cargo.toml
    if !errorlevel! neq 0 (
        set results[1].result=1
    )

    cargo +nightly run --package abi-gen --manifest-path examples\asset\%%f\Cargo.toml
    if !errorlevel! neq 0 (
        set results[2].result=1
    )

    cargo +nightly build --release --no-default-features --features "gm" --target=wasm32-unknown-unknown --manifest-path examples\asset\%%f\Cargo.toml
    if !errorlevel! neq 0 (
        set results[3].result=1
    )
)

