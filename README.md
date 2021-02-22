# BitFormat

Formats bit-focused data structurs for printing to the console or inclusion in log files.

## Warning - Very Early Release

I created this crate to solve a specific problem. It is in a very early stage.
*THERE WILL BE BUGS*

## Changelog

v0.0.1 - Initial crate version
v0.0.2 - Includes examples in README, adds changelog and roadmap

## Roadmap

v0.0.5 - Support for styles (custom border glyphs, coloring)
v0.1.0 - Support for WORD and DWORD tables _(in addition to QWORD)_

## Specific Formats

The library currently recognizes the following bit-focused formats.

### Qword Table

Used to print bytes formatted as a table of QWORDs (32-bit) (8 bytes per row).

#### Example

       +--------+--------+--------+--------+--------+--------+--------+--------+
 Bytes | Byte 0 | Byte 1 | Byte 2 | Byte 3 | Byte 4 | Byte 5 | Byte 6 | Byte 7 |
+------+--------+--------+--------+--------+--------+--------+--------+--------+
|QWORD |10000001|10000011|01011010|00001110|10010001|00110110|00111011|01101100|
|  1   |   (129)|   (131)|    (90)|    (14)|   (145)|    (54)|    (59)|   (108)|
+------+--------+--------+--------+--------+--------+--------+--------+--------+
|QWORD |11110010|
|  2   |   (242)|
+------+--------+

### WebSocket Data Frame

Formats WebSocket data frames as specified in RFC6455:
https://tools.ietf.org/html/rfc6455#section-5.2

#### Example

               +---------------+---------------+---------------+---------------+
  Frame Data   |    Byte  0    |    Byte  1    |    Byte  2    |    Byte  3    |
   (Masked)    +---------------+---------------+---------------+---------------+
   (Short)     |0              |    1          |        2      |            3  |
               |0 1 2 3 4 5 6 7|8 9 0 1 2 3 4 5|6 7 8 9 0 1 2 3|4 5 6 7 8 9 0 1|
       +-------+-+-+-+-+-------+-+-------------+-------------------------------+
       | DWORD |1|0|0|0|0 0 0 1|1|0 0 0 0 0 1 1|0 1 0 1 1 0 1 0|0 0 0 0 1 1 1 0|
       |   1   |F|R|R|R|       |M|             |                               |
       |       |I|S|S|S|op code|A| Payload len |     Masking-key (part 1)      |
       |       |N|V|V|V| (4 b) |S|  (7 bits)   |           (16 bits)           |
       |       | |1|2|3|       |K|             |                               |
       +-------+-+-+-+-+-------+-+-------------+-------------------------------+
       | DWORD |1 0 0 1 0 0 0 1|0 0 1 1 0 1 1 0|0 0 1 1 1 0 1 1|0 1 1 0 1 1 0 0|
       |   2   |                               |  (59)      MASKED  (108)      |
       |       |     Masking-key (part 2)      |0 1 1 0 0 0 0 1|0 1 1 0 0 0 1 0|
       |       |           (16 bits)           |  (97) 'a' UNMASKED  (98) 'b'  |
       |       |                               |     Payload Data (part 1)     |       
       +-------+-------------------------------+-------------------------------+
       | DWORD |1 1 1 1 0 0 1 0|
       |   3   | (242)     MSK |
       |       |0 1 1 0 0 0 1 1|
       |       |  (99) 'c' UNM |
       |       | Payload pt 2  |
       +-------+---------------+
