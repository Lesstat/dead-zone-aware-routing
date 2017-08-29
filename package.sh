#! /bin/bash

ARCHIVE="dzr.tar"
rm -f $ARCHIVE
cargo build --release
tar --transform 's/.*\///g' -cvf $ARCHIVE target/release/dzr 
tar -uvf $ARCHIVE static/*
