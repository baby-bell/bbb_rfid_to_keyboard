#!/usr/bin/env python3
"""
Classes to interface with the pcProx RFID reader.
Authors: Brose Johnstone & previous PCprox.py authors.
"""
import struct
import time
import usb.core

class RFIDReaderUSB(object):
    """ Lower-level (libusb) code to interface with RFID reader """
    def __init__(self, vendor_id=0x0c27, product_id=0x3bfa):
        """
        Try to connect to the RFID reader.
        Throws OSError when things go wrong.
        """
        dev = usb.core.find(idVendor=vendor_id, idProduct=product_id)
        if dev is None:
            raise OSError("Could not find RFID reader device (vid=%d, pid=%d)"
                          % (vendor_id, product_id))
        try:
            dev.detach_kernel_driver(0)
        except usb.USBError:
            pass
        except NotImplementedError:
            pass
        self._dev = dev

    def send_bytes(self, cmd):
        """
        Send CMD to the card reader without reading back a response.
        """
        assert len(cmd) == 8, "Must send only 8 bytes"
        #feature report out, id = 0
        self._dev.ctrl_transfer(0x21, 0x09, 0x0300, 0, cmd)

    def exchange_bytes(self, cmd):
        """
        Send CMD to the card reader and read 8 bytes of reply back.
        """
        assert len(cmd) == 8, "Must send only 8 bytes"
        #feature report out, id = 0
        self._dev.ctrl_transfer(0x21, 0x09, 0x0300, 0, cmd)
        #feature report in, id = 1
        return self._dev.ctrl_transfer(0xa1, 0x01, 0x0301, 0, 8)


class RFIDReader(object):
    """
    Higher-level interface to the reader.
    """
    COMMAND_GET_CARD_ID                = b"\x8f\x00\x00\x00\x00\x00\x00\x00"
    COMMAND_GET_CARD_ID_32             = "\x8d%c\x00\x00\x00\x00\x00\x00"
    COMMAND_GET_CARD_ID_EXTRA_INFO     = "\x8e\x00\x00\x00\x00\x00\x00\x00"
    COMMAND_BEEP_FIRST                 = b"\x8c\x03"
    COMMAND_BEEP_SECOND                = b"\x00\x00\x00\x00\x00"
    COMMAND_GET_MODEL                  = "\x8c\x01%c\x00\x00\x00\x00\x00"
    COMMAND_CONFIG_FIELD_82            = b"\x82\x00\x00\x00\x00\x00\x00\x00"
    COMMAND_GET_VERSION                = "\x8a\x00\x00\x00\x00\x00\x00\x00"

    def __init__(self, low_level):
        """
        Construct an RFIDReader that uses LOW_LEVEL to interface
        with the reader.
        """
        assert not low_level is None, "you must supply a low level interface"
        self._ll = low_level

    def get_card_id(self):
        """
        Fetch a card id from the reader. The id may be up to 8 bytes long.
        """
        card_id = self._ll.exchange_bytes(RFIDReader.COMMAND_GET_CARD_ID)[::-1]
        return [x for x in card_id]

    def get_card_id_32(self):
        """
        Fetch a card id that may be up to 32 bytes long.
        """
        def get_card_id_32_internal(ll, i):
            card_id = ll.exchange_bytes(RFIDReader.COMMAND_GET_CARD_ID_32 % i)[::-1]
            return [x for x in card_id]
        id32_0 = get_card_id_32_internal(self._ll, 0)
        id32_1 = get_card_id_32_internal(self._ll, 1)
        id32_2 = get_card_id_32_internal(self._ll, 2)
        id32_3 = get_card_id_32_internal(self._ll, 3)
        return id32_3 + id32_2 + id32_1 + id32_0


    def beep(self, num_beeps=1, long_beeps=False):
        """
        Make the reader beep NUM_BEEPS times.
        Beep length depends on the state of LONG_BEEPS.
        """
        assert num_beeps >= 1 and num_beeps <= 7, "Beeps must be between 1 and 7"
        num_beeps = bytes([num_beeps + (0x80 if long_beeps else 0x0)])
        self._ll.exchange_bytes(self.COMMAND_BEEP_FIRST +\
        num_beeps + self.COMMAND_BEEP_SECOND)

    def get_model(self):
        """
        Get the model from the card reader. May be padded with null bytes.
        """
        def get_model_internal(ll, i):
            card_id = ll.exchange_bytes(RFIDReader.COMMAND_GET_MODEL % i)
            return [x for x in card_id]
        model1 = get_model_internal(self._ll, 0)
        model2 = get_model_internal(self._ll, 1)
        model3 = get_model_internal(self._ll, 2)
        return model1 + model2 + model3

    def get_additional_id_info(self):
        """
        Return a word (16 bits) of data somehow associated with the card scan.
        Purpose unclear; may be card type. Only works after get_card_id.
        """
        card_id = self._ll.exchange_bytes(RFIDReader.COMMAND_GET_CARD_ID_EXTRA_INFO)
        return struct.unpack("<H", ''.join((chr(x) for x in card_id[:2])))[0]

    def set_led_auto(self):
        """
        Give the reader control over the LED.
        """
        old_data = self._ll.exchange_bytes(RFIDReader.COMMAND_CONFIG_FIELD_82)
        old_data[0] = 0
        old_data[1] = old_data[1] & 0xFD
        self._ll.send_bytes(RFIDReader.COMMAND_CONFIG_FIELD_82)
        self._ll.send_bytes(old_data)

    def set_led_state(self, red, green):
        """
        Manually set the LED to a particular color.
        Setting both RED and GREEN gives amber.
        """
        old_data = self._ll.exchange_bytes(RFIDReader.COMMAND_CONFIG_FIELD_82)
        old_data[0] = ((1 if red else 0) | (2 if green else 0))
        old_data[1] = old_data[1] | 0x02
        self._ll.send_bytes(RFIDReader.COMMAND_CONFIG_FIELD_82)
        self._ll.send_bytes(old_data)

    def get_version(self):
        """
        Get version information about the reader.
        The first two bytes are LUID. The next two bytes are the firmware version.
        The other bytes are unknown.
        """
        ver = self._ll.exchange_bytes(RFIDReader.COMMAND_GET_VERSION)
        return [x for x in ver]


def hexlify(byte_array):
    """
    Turn BYTE_ARRAY into a hexadecimal string.
    """
    return ''.join(['{:02x}'.format(b) for b in byte_array])


def hex_card_id(rdr):
    """
    Returns the full id of the card on RDR in hexadecimal, or None if no card is present.
    """
    try:
        hex_id_str = hexlify(rdr.get_card_id())
        if hex_id_str == '0000000000000001' or hex_id_str == '0000000000000000':
            return None
        return hex_id_str
    except usb.core.USBError:
        return None # this is (hopefully) only a temporary error, so can just ignore it

def wait_until_card(rdr, timeout=3600):
    """
    Waits until a card is swiped or TIMEOUT seconds are reached.
    Default timeout is 1 hour.
    Returns:
        None if the timeout is reached, otherwise the hexadecimal
        id of the card.
    """
    wait_until_none(rdr)
    test_id = hexlify(rdr.get_card_id())
    if test_id == '0000000000000001':
        linux = False # this is the code for no card on Windows
    else:
        linux = True # the code for Linux is 0000000000000000
    card_id = None
    start, curr = time.time(), time.time()
    while not card_id and curr - start < timeout:
        card_id = hex_card_id(rdr)
        curr = time.time()
    if not card_id:
        return None
    if linux:
        card_id = card_id[2:] + '01' # convert to Windows-style number
    return card_id

def wait_until_none(rdr):
    """
    Wait until there is no card on RDR, then return.
    """
    card_id = hex_card_id(rdr)
    while card_id:
        card_id = hex_card_id(rdr)

if __name__ == "__main__":
    rdr = RFIDReader(RFIDReaderUSB())

    print('Starting...', flush=True)

    while True:
        card_id = wait_until_card(rdr)
        print('Card on reader: ' + str(card_id), flush=True)
        wait_until_none(rdr)
        print('Card off the reader.', flush=True)
