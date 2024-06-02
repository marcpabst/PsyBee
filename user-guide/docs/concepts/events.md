# Events

Events are raised whenever something interesting happens in the environment, such as a key press, mouse movement, or a touch event. In psychophusics, events are handled by the `Event` class. There are three types of events that can be raised:

- **Input events:** These events are raised when the user interacts with the experiment using the keyboard, mouse, or touch screen.
- **Window events:** These events are raised when the window is resized, moved, or closed.
- **Device events:** These result from external devices, and are independent of the window.

Some external triggers can result in multiple events being raised at the same time. For example, a moving a physical mouse can result in both an input event and a device event being raised. This is intentional, as one pertains to the movement of the *cursor* and the other to the movement of the *device* (crucially, your operating system might apply transformations to the cursor movement, such as acceleration or inversion, which are not reflected in the device movement). Similarly, a key press can result in both an input event and a device event being raised (but note that windows will only receive input events if they are focused, while device events are independent of focus). It always depends on the specific requirements of your experiment which events you want to handle. 

## Handling events

There are two ways to handle events in PsyBee: polling and callbacks. Polling is the process of checking for events at regular intervals, while callbacks are functions that are called when an event occurs. Both methods have their advantages and disadvantages, and the best method to use depends on the specific requirements of your experiment.

- **Polling:** You can poll for events by creating a new `EventReceiver` object and calling its `poll` method. This will return a list of all events that have occurred since the last call to `poll`. This method is useful when you need to handle events in a loop.
- **Callbacks:** By adding event handlers through `add_event_handler` to a `Window` (or, with certain limitations, to a `Stimulus`), you can register a callback function that will be called whenever an event occurs. This method is useful when you need to handle events as they occur, rather than in a loop.


## Polling for events

... some text here ...

!!! tip
    You can create as many `EventReceiver` objects as you like, and all of them will receive the same events independently of each other and in the same order. This can be useful if you want to handle events in different parts of your experiment in different ways. Note that currently, only 10.000 events are stored in the event queue, so if you don't poll for events for a long time, you might miss some events.

Examples:

```python

event_receiver = window.create_event_receiver()

while True:
    for event in event_receiver.poll():
        if event.type == EventType.KeyPressed:
            print(f"Key {event.key} was pressed")

```



## Handling events with callbacks

!!! warning
    When using callbacks, make sure to keep the callback functions as short as possible. This is especially important when using the Python API, as long-running callback functions can lock the [GIL](https://wiki.python.org/moin/GlobalInterpreterLock) and prevent other parts of the experiment from running.

A callback is a function that is called when an event occurs. You can register a callback function using the `add_event_handler` method on a `Window` object. You can also register a callback function on a `Stimulus` object, but certain limitations apply (see below). The callback function should take a single argument, which is an `Event` object. The `Event` object contains information about the event that occurred, such as the type of event, the key that was pressed, or the position of the mouse.

### Adding event handlers to windows

You can add event handlers to windows by calling the `add_event_handler(kind, callback)` method on a `Window` object. The `kind` argument should be one of the values from the `EventKind` enum, and the `callback` argument should be a function that takes an `Event` object as its only argument. The callback function will be called as soon as possible whenever an event of the specified kind occurs.

Examples:

```python
def on_key_pressed(event):
    if event.key == Key.Enter:
        # print a message when the "Enter" key is pressed
        print("Enter key was pressed")

# register the callback function
window.add_event_handler(EventType.KeyPressed, on_key_pressed)
```

### Adding event handlers to stimuli

You can also add event handlers to stimuli, but because it is hard to correlate frame-by-frame stimulus presentation with precise event timing, this only works when stimuli are added directly to the window (and not to a frame). Nonetheless, this can be useful in some cases, for example when you want to handle events that are not extremely time-sensitive (like clicks on a button).

Examples:

```python
# Create a button stimulus
button = psy.ButtonStimulus(window, psy.Rectangle(100, 100, 200, 200))

# Make the button change color when it is hovered over
def on_button_pressed(event):
    button.set_color(psy.Color(1, 0, 0))

button.add_event_handler(EventType.MouseButtonPressed, on_button_pressed)

window.add_stimulus(button)
```

!!! note
    For positional stimuli, the list of stimuli is traversed in the reverse order of their addition to the window. This means that the last stimulus added to the window will be the first to receive events (as it is "on top" of all other stimuli). On an implementation level, stimuli then can "decide" whether to "consume" an event or pass it on to the next stimulus in the list. This is useful to make some stimuli "invisible" to certain events, or to make some stimuli "consume" an event so that it is not passed on to other stimuli.

