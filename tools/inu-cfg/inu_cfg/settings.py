from .validator import Validator

CSV_DATA = """key,type,encoding,value
settings,namespace,,
clock,data,u16,{}
device_id,data,string,"{}"
wifi_ap,data,string,"{}"
wifi_pw,data,string,"{}"
"""


class Settings:
    def __init__(self, clk, dvc_id, ssid, pw):
        self.clock = clk
        self.device_id = dvc_id
        self.ssid = ssid
        self.password = pw

    @staticmethod
    def from_validator(v: Validator):
        return Settings(v.clock, v.device_id, v.ssid, v.password)

    def write(self, filename):
        with open(filename, 'w') as file:
            file.write(CSV_DATA.format(
                self.clock,
                self.device_id,
                self.ssid,
                self.password
            ))
        print("Table data writen to {}".format(filename))
