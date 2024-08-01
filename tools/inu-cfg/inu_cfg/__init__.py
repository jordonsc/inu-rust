import os
import esptool

from .validator import Validator
from .settings import Settings
from esp_idf_nvs_partition_gen import nvs_partition_gen

INPUT_FN = "nvs.csv"
OUTPUT_FN = "nvs.bin"
PARTITION_OFFSET = 0x9000
PARTITION_SIZE = 0x4000
BAUD = 115200
PREFERRED_PORT = "/dev/ttyACM0"


class GenArgs:
    def __init__(self):
        self.version = 2
        self.size = f"{PARTITION_SIZE}"
        self.input = INPUT_FN
        self.output = OUTPUT_FN
        self.outdir = "."
        self.keyfile = None
        self.key_protect_hmac = None
        self.kp_hmac_inputkey = None

    def __str__(self):
        return str(self.__class__) + ": " + str(self.__dict__)


class EspArgs:
    def __init__(self):
        self.chip = 'auto'
        self.port = '/dev/ttyACM0'
        self.baud = 115200
        self.before = 'default_reset'
        self.after = 'hard_reset'
        self.no_stub = False
        self.trace = False
        self.override_vddsdio = None
        self.connect_attempts = 2
        self.operation = 'write_flash'
        self.addr_filename = []
        self.erase_all = False
        self.flash_freq = 'keep'
        self.flash_mode = 'keep'
        self.flash_size = 'keep'
        self.spi_connection = None
        self.no_progress = False
        self.verify = False
        self.encrypt = False
        self.encrypt_files = None
        self.ignore_flash_encryption_efuse_setting = False
        self.force = False
        self.compress = None
        self.no_compress = False

    def __str__(self):
        return str(self.__class__) + ": " + str(self.__dict__)


class NvsGenerator:
    def __init__(self, cli_args, enc=False):
        self.cli_args = cli_args
        self.gen_args = GenArgs()
        self.esp_args = EspArgs()
        self.esp_args.port = self.cli_args.port
        self.encryption = enc

    def generate(self):
        v = Validator.from_args(self.cli_args)
        v.validate()

        s = Settings.from_validator(v)
        s.write(INPUT_FN)

        nvs_partition_gen.generate(self.gen_args, is_encr_enabled=self.encryption)

    def flash(self):
        if self.esp_args.port:
            serial_list = [self.esp_args.port]
        else:
            serial_list = esptool.get_port_list()

            if PREFERRED_PORT in serial_list:
                serial_list.remove(PREFERRED_PORT)
                serial_list.append(PREFERRED_PORT)

        with open(OUTPUT_FN, "rb") as fp:
            self.esp_args.addr_filename = [(PARTITION_OFFSET, fp)]

            esp = esptool.get_default_connected_device(
                serial_list,
                port=self.esp_args.port,
                connect_attempts=self.esp_args.connect_attempts,
                initial_baud=BAUD,
                chip=self.esp_args.chip,
                trace=self.esp_args.trace,
                before=self.esp_args.before,
            )

            if not esp:
                print("\nUnable to connect to ESP device")
                return

            esp = esp.run_stub()
            esptool.write_flash(esp, self.esp_args)

    @staticmethod
    def clean():
        try:
            os.remove(INPUT_FN)
        except Exception as e:
            print(f"Cannot delete {INPUT_FN}: {e}")

        try:
            os.remove(OUTPUT_FN)
        except Exception as e:
            print(f"Cannot delete {OUTPUT_FN}: {e}")
