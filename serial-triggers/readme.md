# SerialWriter

Send integers to serial ports with precise timing control. Ideal for sending "trigger" signals to external devices.

## Usage

```python
from serial_triggers import SerialTriggerWriter

# List ports
ports = SerialTriggerWriter.list_ports()

# Basic usage
writer = SerialTriggerWriter("/dev/ttyUSB0", 9600)
writer.write(42)
writer.write(43, delay=0.1)
```
