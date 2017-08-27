#! /bin/bash

rm -f flos_fapra.tar
cargo build --release
tar --transform 's/.*\///g' -cvf flos_fapra.tar target/release/fapra 
tar -uvf flos_fapra.tar static/*
