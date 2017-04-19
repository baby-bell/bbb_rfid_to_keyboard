from reader_interface_py3 import RFIDReader, RFIDReaderUSB, wait_until_card
from emukb_python import Keyboard
from sys import exit
from time import sleep

def main():
    logfile = open("/home/ubuntu/fail.txt", "w")

    while True:
        try:
            reader = RFIDReader(RFIDReaderUSB())
            break
        except Exception:
            sleep(0.25)
            # ignore exception, hopefully it's just temporary

    try:
        keyboard = Keyboard()

        while True:
            card_id = wait_until_card(reader)
            keyboard.send_string(str(card_id))
    except Exception as e:
        logfile.write(str(e))
        exit(1)


if __name__ == "__main__": main()
