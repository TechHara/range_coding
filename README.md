This repo contains simple data compression utility based on range coding for educational purposes. The encoder/decoder implementation is based on [Wikipedia](https://en.wikipedia.org/wiki/Range_coding) exxample.

## Build
```bash
cargo build -r
```

## Encoder
```bash
# encode linux.tar and output to linux.tar.enc
target/release/encode < linux.tar > linux.tar.enc

# encode linux.tar with verbose output for debugging purposes
# this will run extremely slow
target/release/encode 1 < linux.tar > linux.tar.enc 2> encode.log
```

## Decoder
```bash
# decode linux.tar.enc and output to linux.tar.dec
target/release/decode < linux.tar.enc > linux.tar.dec

# decode linux.tar.enc with verbose output for debugging purposes
# this will run extremely slow
target/release/decode 1 < linux.tar.enc > linux.tar.dec 2> decode.log
```

## Verify
```bash
# make sure encoder-decoder cycle reproduces the original data
cmp linux.tar linux.tar.dec
```