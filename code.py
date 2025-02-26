import board
import digitalio
import busio
import time

cs = digitalio.DigitalInOut(board.RX)
cs.direction = digitalio.Direction.OUTPUT
cs.value = True
spi = busio.SPI(board.SCK, board.MOSI, board.MISO)
while not spi.try_lock():
    pass

spi.configure(baudrate=1000000, phase=0, polarity=0)

def spi_send(v):
    cs.value = False
    spi.write(bytes(v))
    cs.value = True

# Reset by setting mode register to 0
spi_send([0x00])
# Set ID
spi_send([0x06, 0x57, 0x5a, 0x52, 0x46])
# Mode Control to 0x42 (Auto RSSI measurement while entering RX mode, FIFO mode)
spi_send([0x01, 0x42])

# RC OSC Register 3
#  BBCKS = 0, F_SYCK/8 as recommended
spi_send([0x09, 0x05])

# Goto standby mode
spi_send([0xa0])

# GPIO to MISO
spi_send([0x0b, 0b00011001])
spi_send([0x0c, 0x01])

# Clock register
#  GRC = 0, F_XTAL x (DBL+1) / (GRC+1) = 2M when CGS == 1, (since it's 0, it's do-not-care)
#  CSC = 1, system clock F_SYCK divider select, F_SYCK = F_MCLCK / 2
#  CGS = 0, disables internal 32MHz PLL clock
#  XS = 1, Crystal oscillator select is 1 meaning, crystal, not external clock
spi_send([0x0d, 0x05])

# Data rate = F_SYCK / 32 / {SDR + 1}. SDR is 0 here so it's just F_SYCK / 32
spi_send([0x0e, 0x00])

#### PLL
# Pll 1
# Channel = 0
spi_send([0x0f, 0x00])

# Pll 2
# DBL = 1
# RRC = 0
# CHR = 15
# BIP 8 = 0
spi_send([0x10, 0x9e])

# Pll 3
# BIP = 0x4b = 75
spi_send([0x11, 0x4b])

# Pll 4
# BFP 15 -> 8 = 0
spi_send([0x12, 0x00])

# Pll 5
# BFP = 2
spi_send([0x13, 0x02])
# F_LO_BASE = F_PFD * (BIP + BFP/2^16) = (DBL + 1) * F_XTAL/(RRC+1) * (BIP + BFP/2^16)
#                                      = 2         * 16Mhz /1       * (75  +   2/2^16)
#                                      = 32 Mhz * (75 + 2^-15)
#           = 2400.0009765625 ~= 2400.001 Mhz
# F_CHSP = F_XTAL * (DBL+1) / 4 / (CHR+1)
#        = 16Mhz  * 2       / 4 / (15 + 1)
#        = 8Mhz / 16
#        = 500 Khz

# Tx 1 and 2
spi_send([0x14, 0x16])
spi_send([0x15, 0x2b])

# Delay 1 and 2
spi_send([0x16, 0x12])
spi_send([0x17, 0x40])

# Rx
# RXSM = 3, FC = 0, RXDI = 0, DMG = 0, BWS = 1, ULS = 0
spi_send([0x18, 0x62])

# Rx gain 1 thru 4
spi_send([0x19, 0x80])
spi_send([0x1a, 0x80])
spi_send([0x1b, 0x00])
spi_send([0x1c, 0x0a])

# Rssi Threshold
spi_send([0x1d, 0x32])

# Adc
spi_send([0x1e, 0xc3])

# Code 1 thru 3
spi_send([0x1f, 0x07])
spi_send([0x20, 0x16])
spi_send([0x21, 0x00])

# If calibration 1
spi_send([0x22, 0x00])

# Vco current calibration
spi_send([0x24, 0x00])

# Vco single band calibration 1 thru 2
spi_send([0x25, 0x00])
spi_send([0x26, 0x3b])
spi_send([0x27, 0x00])
spi_send([0x28, 0x17])
spi_send([0x29, 0x47])
spi_send([0x2a, 0x80])
spi_send([0x2b, 0x03])
spi_send([0x2c, 0x01])
spi_send([0x2d, 0x45])
spi_send([0x2e, 0x18])
spi_send([0x2f, 0x00])
spi_send([0x30, 0x01])
spi_send([0x31, 0x0f])

# Strobe command - Pll mode
spi_send([0xb0])

def wait_for_calibration():
    print("Calibrating")
    calibrated = False
    while not calibrated:
        cs.value = False
        result = bytearray(2)
        spi.write_readinto(bytes([0x42, 0x00]), result)
        cs.value = True
        calibrated = result[1] == 0
        print(".")
    print("Calibrated")

spi_send([0x02, 0x01])
wait_for_calibration()

cs.value = False
result = bytearray(2)
spi.write_readinto(bytes([0x62, 0x00]), result)
cs.value = True
print("IF Calibration Value = ", result[1], "| Pass = ", result[1] & 0b10000 == 0)

spi_send([0x24, 0x13])
spi_send([0x26, 0x3b])

spi_send([0x0f, 0x00])
spi_send([0x02, 0x02])
wait_for_calibration()

cs.value = False
result = bytearray(2)
spi.write_readinto(bytes([0x65, 0x00]), result)
cs.value = True
print("VcoSingleBandCalibration1 Value = ", result[1], "| Pass = ", result[1] & 0b1000 == 0)

spi_send([0x0f, 0xa0])
spi_send([0x02, 0x02])
wait_for_calibration()

cs.value = False
result = bytearray(2)
spi.write_readinto(bytes([0x65, 0x00]), result)
cs.value = True
print("VcoSingleBandCalibration1 Value = ", result[1], "| Pass = ", result[1] & 0b1000 == 0)

# Goto standby mode
spi_send([0xa0])

# TX Test register
# TX Current Setting = 0
# PAC is 3
# TX Buffer setting is 7
# Basically max power (1.3 dBm, current at 21.25 mA)
spi_send([0x28, 0x1f])

# Data rate register, sets Data rate to F_SYCK / 32 / (*1* + 1)
spi_send([0x0e, 0x01])

# Mode control register
# ARSSI (Auto RSSI measurement while entering RX mode) and AIF (Auto IF Offset) enabled
# Sets FIFO Mode
spi_send([0x01, 0x62])

# Code Register
# Enables CRC calculation and transmission during TX
# Sets ID code length to be 4 bytes, and preamble length to be 4 bytes
# Disables data whitening, and forward error correction
spi_send([0x1f, 0x0f])

# PLL Register
# Sets LO channel number to 4
# F_LO = F_LO_BASE + F_OFFSET = F_LO_BASE + 4 * F_CHSP
#     ~= 2400.001 Mhz + 4 * 500 Khz
#     ~= 2402.001 Mhz
spi_send([0x0f, 0x04])

# Set up "Easy FIFO Mode"
# Sets FIFO length to 16 bytes, 0s out PSA and FPM
spi_send([0x03, 0x0f])
spi_send([0x04, 0x00])

# Goto standby mode
spi_send([0xa0])

# Fifo write pointer reset
spi_send([0xe0])

# Set up message
spi_send([0x05, 0x45, 0x20, 0x21, 0x64, 0x21, 0x34, 0x21, 0x14, 0x2c, 0x14, 0x2c, 0x14, 0x2c, 0x14, 0x21, 0x00])

gpio2 = digitalio.DigitalInOut(board.TX)
gpio2.direction = digitalio.Direction.INPUT
# goto tx mode
spi_send([0xd0])

while gpio2.value != 0:
    print("Waiting for TX")

list = [100, 99, 97, 92, 87, 80, 72, 64, 56, 48, 40, 33, 28, 23, 21, 20, 21, 23, 28, 33, 40, 48, 56, 64, 72, 80, 87, 92, 97, 99]
list_len = len(list)
i = 0
print("entering loop")
while True:
    light_a = list[i]
    light_b = list[(i + list_len // 3) % list_len]
    light_c = list[(i + 2 * list_len // 3) % list_len]

    spi_send([0xe0])
    spi_send([0x05, 0x47, light_a, 0x21, light_b, 0x21, light_c, 0x21, 0x14, 0x2c, 0x14, 0x2c, 0x14, 0x2c, 0x14, 0x21, 0x00])
    spi_send([0xd0])

    while gpio2.value != 0:
        pass

    i = (i + 1) % list_len
    time.sleep(0.03)

spi.unlock()
while True:
    pass
