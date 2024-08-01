#!/usr/bin/env python3

import argparse
from inu_cfg import NvsGenerator

DEFAULT_CLOCK = "160"

parser = argparse.ArgumentParser(description='Inu Ferric Configurator')

parser.add_argument('-c', '--clock', dest='clock', action='store',
                    help='MCU clock speed in MHz', default=DEFAULT_CLOCK)
parser.add_argument('-d', '--device_id', dest='device_id', action='store', help='Inu device ID')
parser.add_argument('-s', '--ssid', dest='ssid', action='store', help='WiFi SSID')
parser.add_argument('-x', '--password', dest='password', action='store', help='WiFi password')
parser.add_argument('-p', '--port', dest='port', action='store', help='Port to ESP32 device')

if __name__ == '__main__':
    args = parser.parse_args()

    gen = NvsGenerator(args)
    gen.generate()
    gen.flash()
    gen.clean()
