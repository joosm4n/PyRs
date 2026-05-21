#!/bin/sh
cd ../

if command -v rustup &> /dev/null
then
    rustup --version
    echo "rustup installed"

else 
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.sh | sh
    rustup toolchain install stable

if command -v cargo &> /dev/null
then
    
    cargo --version    
    echo "cargo installed"

else
    echo "cargo is not installed"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    rustup toolchain install stable

if command -v gcc &> /dev/null
then
    echo "GCC C compiler is installed."
    echo "Version information:"
    gcc --version
    
else
    apt update
    apt install build-essential
    apt install libgmp-dev libmpfr-dev
fi
l
if command -v m4 &> /dev/null
then 
    echo "m4 is installed"
    m4 --version

else
    apt install m4

fi

cargo build


