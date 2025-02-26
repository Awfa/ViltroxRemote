```
Power off           0x[42, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Power on            0x[43, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
                        ^ 8th bit changees, 0 for off, 1 for on -- amendment: turns out only for group a

Brightness up 1     0x[43, 21, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Brightness up 2     0x[43, 22, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Brightness up 3     0x[43, 23, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Brightness down 1   0x[43, 22, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Brightness down 2   0x[43, 21, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Brightness down 3   0x[43, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
                           ^^ brightness byte?

Temperature up 1    0x[43, 20, 22, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Temperature up 2    0x[43, 20, 23, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Temperature up 3    0x[43, 20, 24, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Temperature down 1  0x[43, 20, 23, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Temperature down 2  0x[43, 20, 22, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Temperature down 3  0x[43, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
                               ^^ temperature byte?
```
=================
```
Power on - Group A              0x[49, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Power off - Group A             0x[48, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Power on - Group B              0x[4a, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Power off - Group B             0x[48, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Power on - Group C              0x[4c, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Power off - Group C             0x[48, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Power on/off - Group D          0x[40, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Power on/off - Group D          0x[48, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Power on/off - Group E          0x[58, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Power on/off - Group E          0x[48, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Power on/off - Group F          0x[68, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Power on/off - Group F          0x[48, 20, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
```
First byte contains power for all 6 lights?

In binary: 01FE DCBA, if A..F bit is 1, then the light is 'on'

=================
```
Brightness/Temp Min Group A     0x[49, 14, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Brightness Maxed Group A        0x[49, 64, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Temperature Maxed Group A       0x[49, 64, 38, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
                                       ^^  ^^ brightness and temperature (respectively), taking up a whole byte each
```
Brightness min is 0x14 (20 decimal). Matches 20% brightness min.
Brightness max is 0x64 (100 decimal). Matches 100% brightness max.
Brightness is just brightness percent as an integer, ranging from [20, 100].

Temperature min is 0x21 (33 decimal). Matches 3300k color temp min.
Temperature max is 0x38 (56 decimal). Matches 5600k color temp max.
Temperature is just the integer divided by 100 then, ranging from [33, 56].
=================
```
                                0x[49, 14, 21, 64, 21, 34, 21, 14, 2c, 14, 2c, 14, 2c, 14, 21, 00]
Hypothesized data format           PWR B/A T/A B/B T/B B/C T/C B/D T/D B/E T/E B/F T/F B/G T/G END

PWR, In binary: 01FE DCBA, if A..F bit is 1, then the light is 'on'
B/x, brightness for group x, a byte ranging from 20 - 100 (represents 20% to 100% brightness).
T/x, temperature for group x, a byte ranging from 33 - 56 (represents 3300K - 5600K).

There seems to be room for a group G in the message protocol, but the values are just at the lowest.
END, just a 0 byte
```