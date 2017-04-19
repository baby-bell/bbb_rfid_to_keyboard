# Prior Configuration on the Beaglebone Black
- You need to remove the `g_multi` and `g_ether` modules
In `/etc/modprobe.d/local.conf`, write
```
install g_ether /bin/true
install g_multi /bin/true
```
Also remove the USB networking device from `/etc/network/interfaces`
- Create the directory `/config`

# Commands to run before executing the Python script
```
sudo modprobe usb_f_hid
sudo mount -t configfs none /config
```

