import ptest
import time


def handler(event):
    print("Got event: ", event)


ptest.add_event_handler(handler)

for i in range(1000):
    print(i)
    time.sleep(0.5)
