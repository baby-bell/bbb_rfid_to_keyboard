#!/usr/bin/env python3
"""
A python interface to the Beaglebone emulated keyboard.

Author: Brose Johnstone
"""
import ctypes
from ctypes import cdll, byref, POINTER, c_char_p as cstr

class EmulatedKb(ctypes.Structure):
    pass

class KeyboardError(ctypes.Structure):
    """ A wrapper class for a description of errors
    that the keyboard can encounter. """
    _fields_ = [("description", cstr)]
    def __del__(self):
        if hasattr(self, "description"):
            error_free(self)


class Keyboard(object):
    """An emulated keyboard encapsulated in a Python object.

    Usage:
    Call the constructor. If no exception is thrown,
    you can send a string on the keyboard using
    `send_string`. Python will take care of resource
    management and will automagically release the
    keyboard when the program ends.
    """
    def __init__(self):
        """Attempt to acquire a handle to an emulated
        keyboard.
        """
        self.kb = POINTER(EmulatedKb)()
        error = KeyboardError()
        if not init_kb(byref(self.kb), byref(error)):
            raise OSError("{}".format(error.description.decode()))

    def send_string(self, msg):
        """ Make the keyboard type out `msg`. """
        error = KeyboardError()
        if not send_string(self.kb, msg.encode('utf-8'), byref(error)):
            err_msg = "unable to send string: {}".format(error.description.decode())
            raise OSError(err_msg)

    def __del__(self):
        if hasattr(self, "kb"):
            deinit_kb(self.kb)

# Load shared object
emukb = cdll.LoadLibrary("/home/ubuntu/librfid_to_kb.so")
# Set up function prototypes & return values
init_kb = emukb.emukb_init
init_kb.argtypes = [POINTER(POINTER(EmulatedKb)), POINTER(KeyboardError)]
init_kb.restype = ctypes.c_bool

deinit_kb = emukb.emukb_deinit
deinit_kb.argtypes = [POINTER(EmulatedKb)]
deinit_kb.restype = None

send_string = emukb.emukb_send_string
send_string.argtypes = [POINTER(EmulatedKb), cstr, POINTER(KeyboardError)]
send_string.restype = ctypes.c_bool

error_free = emukb.error_free
error_free.argtypes = [KeyboardError]
error_free.restype = None
