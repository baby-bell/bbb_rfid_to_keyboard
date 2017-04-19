# Convert input from RFID reader to Keyboard for BeagleBone Black
This project requires `libusb` to read from the RFID reader and 
kernel >= 4.0.0 on the BeagleBone, for USB HID gadget support.

# Documentation on other efforts
 - BeagleBone KVM [here](https://hacks.pmf.io/2015/06/24/the-beaglebone-black-as-a-smart-kvm/)
 - HID Gadget API dox (https://www.kernel.org/doc/Documentation/usb/gadget_hid.txt)
 - Gadgets thru configfs http://lxr.free-electrons.com/source/Documentation/usb/gadget_configfs.txt

# Basic Process for USB Gadget Configuration
1. Mount the configfs (TODO: is this necessary?)
2. Make a directory for your USB gadget
3. Configure data about the gadget (vendor, product, etc)
4. Write the binary device descriptor