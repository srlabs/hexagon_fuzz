# iOS Firmware and Extraction
This will guide through the steps for obtaining the iOS firmware and extracting the baseband firmware from it. 

## Downloading iPhone IPSW Firmware
1. The iPhone firmware can be downloaded from [ipsw.me](https://ipsw.me), by selecting the targeted model number.

## Extracting the firmware
Steps to extract the firmware:
1. Unzip the *.ipsw file, and you'll get the baseband firmware file with the extension *.bbfw
![alt text](images/file_output_bbfw.png)
2. Unzip the *.bbfw file and you'll find the `qdsp6sw.mbn` file which is the hexagon baseband firmware file.
![alt text](images/file_output_mbn.png)

## Analyse the firmware
The hexagon tools from qualcomm are very helpful in analysing the firmware. 
- hexagon-readelf: can be used to analyze hexagon ELF binaries, it is tailormade for Hexagon DSP architecture. 
```bash
$ hexagon-readelf -all qdsp6sw.mbn
ELF Header:
  Magic:   7f 45 4c 46 01 01 01 00 00 00 00 00 00 00 00 00
  Class:                             ELF32
  Data:                              2's complement, little endian
  Version:                           1 (current)
  OS/ABI:                            UNIX - System V
  ABI Version:                       0x0
  Type:                              EXEC (Executable file)
  Machine:                           Qualcomm Hexagon
  Version:                           0x1
  Entry point address:               0x90000000
  Start of program headers:          52 (bytes into file)
  Start of section headers:          0 (bytes into file)
  Flags:                             0x71 <unknown>
  Size of this header:               52 (bytes)
  Size of program headers:           32 (bytes)
  Number of program headers:         32
  Size of section headers:           40 (bytes)
  Number of section headers:         0
  Section header string table index: 0
There are 0 section headers, starting at offset 0x0:

Section Headers:
  [Nr] Name              Type            Address  Off    Size   ES Flg Lk Inf Al
Key to Flags:
  W (write), A (alloc), X (execute), M (merge), S (strings), l (large)
  I (info), L (link order), G (group), T (TLS), E (exclude), x (unknown)
  O (extra OS processing required) o (OS specific), p (processor specific)

There are no relocations in this file.

Elf file type is EXEC (Executable file)
Entry point 0x90000000
There are 32 program headers, starting at offset 52

Program Headers:
  Type           Offset   VirtAddr   PhysAddr   FileSiz MemSiz  Flg Align
  NULL           0x000000 0x00000000 0x00000000 0x00434 0x00000     OS[0x70] 0x0
  NULL           0x001000 0x9f600000 0x9f600000 0x00630 0x01000     OS[0x22] 0x1000
  LOAD           0x002000 0xfe100000 0x90000000 0x01c24 0x02000 R E OS[0x80] 0x1000
  LOAD           0x004000 0xfe102000 0x90002000 0xc43f4 0xc5000 RWE OS[0x80] 0x1000
  LOAD           0x0c9000 0xfe000000 0x900c7000 0x039a4 0x04000 R E OS[0x80] 0x1000
  LOAD           0x0cd000 0xfe004000 0x900cb000 0x03ce4 0x04000 RW  OS[0x80] 0x1000
  LOAD           0x0d1000 0xfe008000 0x900cf000 0x03178 0x04000 R E OS[0x80] 0x1000
  LOAD           0x0d5000 0xfe00c000 0x900d3000 0x08840 0x09000 RWE OS[0x80] 0x1000
  LOAD           0x0de000 0xc0100000 0x90100000 0x02064 0x03000 RWE OS[0x80] 0x1000
  LOAD           0x0e1000 0xc0110000 0x90110000 0x25bf7c 0x25c000 R E OS[0x80] 0x1000
  LOAD           0x33d000 0xc0370000 0x90370000 0x09140 0x0a000 R   OS[0x80] 0x1000
  LOAD           0x347000 0xc0380000 0x90380000 0x0c000 0x0c000 R E OS[0x80] 0x1000
  LOAD           0x353000 0xc038c000 0x9038c000 0x537cd0 0x538000 R E OS[0x80] 0x1000
  LOAD           0x88b000 0xc0900000 0x90900000 0x20000 0x20000 RW  OS[0x80] 0x1000
  LOAD           0x8ab000 0xc0940000 0x90940000 0x464000 0x464000 RW  OS[0x80] 0x1000
  LOAD           0xd0f000 0xc0da4000 0x90da4000 0x00000 0x01000 RW  OS[0x80] 0x1000
  LOAD           0xd0f000 0xc0da5000 0x90da5000 0x35b090 0x35c000 RW  OS[0x80] 0x1000
  LOAD           0x106b000 0xc1140000 0x91140000 0x73a30 0x74000 RW  OS[0x80] 0x1000
  LOAD           0x10df000 0xc11c0000 0x911c0000 0x38c000 0x38c000 RW  OS[0x80] 0x1000
  LOAD           0x146b000 0xc154c000 0x9154c000 0xd90bcc 0x2ba1000 RW  OS[0x80] 0x1000
  LOAD           0x21fc000 0xc40ed000 0x940ed000 0xf58468 0xf59000 R   OS[0x80] 0x1000
  LOAD           0x3155000 0xc5046000 0x95046000 0xa19008 0xa1a000 RW  OS[0x80] 0x1000
  LOAD           0x3b6f000 0xc5a60000 0x95a60000 0x00000 0x59dd000 RW  OS[0x80] 0x1000
  LOAD           0x3b6f000 0xcb43d000 0x9b43d000 0x1c980 0x1d000 RW  OS[0x80] 0x1000
  LOAD           0x3b8c000 0xcb460000 0x9b460000 0x232e040 0x232f000 RW  OS[0x80] 0x1000
  LOAD           0x5ebb000 0xcd790000 0x9d790000 0x6beec0 0x6bf000 RW  OS[0x80] 0x1000
  LOAD           0x657a000 0xcde80000 0x9de80000 0x1394ea 0x13a000 R   OS[0x80] 0x1000
  LOAD           0x66b4000 0xcdfba000 0x9dfba000 0x00118 0x01000 R   OS[0x80] 0x1000
  LOAD           0x66b5000 0xbfe00000 0x9dfbb000 0x47000 0x47000 RWE OS[0x80] 0x1000
  LOAD           0x66fc000 0xbfe80000 0x9e002000 0x18fc0 0x19000 R E OS[0x80] 0x1000
  LOAD           0x6715000 0xbfea0000 0x9e01b000 0x02a08 0x03000 RW  OS[0x80] 0x1000
  LOAD           0x6718000 0xfe200000 0x9e01e000 0x00000 0x15e2000 R   OS[0x80] 0x1000

 Section to Segment mapping:
  Segment Sections...
   00
   01
   02
   03
   04
   05
   06
   07
   08
   09
   10
   11
   12
   13
   14
   15
   16
   17
   18
   19
   20
   21
   22
   23
   24
   25
   26
   27
   28
   29
   30
   31
```
- The memory map information is helpful to help emulation in placing the code and data in the correct memory regions, it is also helpful when reversing the firmware with Ghidra.
The guide [memory mapping information](reverse_engineering.md#update-memory-map-in-ghidra) walks through the required steps.



