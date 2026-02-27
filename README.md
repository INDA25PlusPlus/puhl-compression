# puhl-compression

Program for compressing data using huffman coding

## Example usage
### Compression
```
$ cat main.rs | cargo run > main.compressed
```

### Decompression
```
$ cat main.compressed | cargo run -- -d > main.rs
```