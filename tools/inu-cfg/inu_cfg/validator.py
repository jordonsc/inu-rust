import re

class Validator:
    DEFAULT_CLOCK = 160

    def __init__(self, clk, dvc_id, ssid, pw):
        self.clock = self.validate_clock(clk)
        self.device_id = self.validate_device_id(dvc_id)
        self.ssid = self.validate_ssid(ssid)
        self.password = self.validate_password(pw)

    @staticmethod
    def from_args(args):
        return Validator(args.clock, args.device_id, args.ssid, args.password)

    def validate(self):
        self.clock = self.validate_clock(self.clock)
        while self.clock is None:
            print(f"MCU clock speed ({self.DEFAULT_CLOCK}): ", end="")
            self.clock = self.validate_clock(input())

        self.device_id = self.validate_device_id(self.device_id)
        while self.device_id is None:
            print("Device ID: ", end="")
            self.device_id = self.validate_device_id(input())

        self.ssid = self.validate_ssid(self.ssid)
        while self.ssid is None:
            print("AP SSID: ", end="")
            self.ssid = self.validate_ssid(input())

        self.password = self.validate_password(self.password)
        while self.password is None:
            print("AP password: ", end="")
            self.password = self.validate_password(input())

    @staticmethod
    def validate_clock(clk):
        c = int(clk if clk else Validator.DEFAULT_CLOCK)
        if c not in [80, 160, 240]:
            print("Clock speed must be 80, 160, or 240 MHz")
            return None
        return c

    @staticmethod
    def validate_device_id(dvc_id):
        if not dvc_id:
            return None

        dvc_id = dvc_id.strip().lower()

        if len(dvc_id) < 3:
            print("Device ID must be at least 3 characters")
            return None

        if not re.match(r'^[a-z0-9\-.]+$', dvc_id):
            print("Device ID can only contain characters a-z, 0-9, hyphen (-), and period (.)")
            return None

        return dvc_id

    @staticmethod
    def validate_ssid(ssid):
        if not ssid:
            return None

        if len(ssid) > 32:
            print("SSID cannot exceed 32 characters")
            return None

        return ssid

    @staticmethod
    def validate_password(pw):
        if pw is None:
            return None

        if len(pw) > 63 or len(pw) < 8:
            print("AP password must be between 8 and 63 characters")
            return None

        return pw
