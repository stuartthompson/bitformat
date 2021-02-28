## Roadmap

* v0.0.6 -> v0.1.0 - Reserved for bug fixes in v0.0.5 and earlier

* v0.1.0 - Cell-based layout rendering
* v0.1.1 - WebSocket data frame medium (16bit) and long (64bit) packet lengths
* v0.1.2 - WebSocket data frame summary
* v0.1.3 - Support for styles in qword table (custom colors)
* v0.1.4 - Support for custom border glyphs
* v0.1.5 - Support for WORD and DWORD tables _(in addition to QWORD)_


## v0.1.0 - Cell Based Layout Rendering

The concept for cell-based layout rendering is to consider each of the tables 
that bitformat output as an iterable collection of cells. Each specific bit 
format provides an iterator that returns cells until it is out of data. A 
master formatter iterates through the data, drawing rows according to the 
number of cells per row for the master format (WORD, DWORD, QWORD, etc...) 
until it has rendered all of the data.