#!/bin/sh
cd ../

if command -v cargo &> /dev/null
then
    cargo --version
    echo "Cargo is installed. Proceeding with script..."

else
    echo "Cargo is not installed. Please install Rust and Cargo."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    rustup toolchain install stable

if command -v gcc &> /dev/null
then
    echo "GCC C compiler is installed."
    echo "Version information:"
    gcc --version
else
    sudo apt update
    sudo apt install build-essential
    sudo apt install libgmp-dev libmpfr-dev

if command -v m4 &> /dev/null
then 
    echo "m4 is installed"
    m4 --version
else
    sudo apt install m4

cargo build


