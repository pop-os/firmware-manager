project('example', 'c')

dbus = dependency('dbus-1')
gtk = dependency('gtk+-3.0')
s76 = dependency('firmware_manager')
threads = dependency('threads')

executable('example', 'main.c', dependencies : [ dbus, gtk, s76, threads ])