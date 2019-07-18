#include <glib.h>

typedef struct { } S76FirmwareWidget;

S76FirmwareWidget *s76_firmware_widget_new (void);

GtkWidget *s76_firmware_widget_container (const S76FirmwareWidget *self);

int s76_firmware_widget_scan (S76FirmwareWidget *self);

void s76_firmware_widget_free (S76FirmwareWidget *self);