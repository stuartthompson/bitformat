# BitFormat

Formats bit-focused data structurs for printing to the console or inclusion in log files.

## Very Early Release

I created this crate for my own use. It is in a very early stage. *THERE WILL BE BUGS*

## Specific Formats

The library currently recognizes the following bit-focused formats.

### Qword Table

Used to print bytes formatted as a table of QWORDs (32-bit) (8 bytes per row).

### WebSocket Data Frame

Formats WebSocket data frames as specified in RFC6455:
https://tools.ietf.org/html/rfc6455#section-5.2