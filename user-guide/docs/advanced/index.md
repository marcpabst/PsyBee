# Advanced Usage

## Event-driven experiment flows

The traditional way to write a PsyBee experiment is to use a loop and submit individual frames to the window. This is simple, provides a high degree of control over stimulus presentation, and is suitable for many experiments. However, while PsyBee is not designed to be a GUI toolkit, it supports some limited event-driven programming.

In short, event-driven programming is a programming paradigm in which the flow of the program is determined by events such as user actions (mouse clicks, key presses), sensor outputs, or messages from other programs or threads. In the context of PsyBee, this means that the flow of the experiment is determined by events such as the start of a trial, the end of a trial, or the presentation of a stimulus.


Event-driven programming is a powerful form of encapsulation that is useful when dealing with complex, asynchronous systems. It is particularly useful when dealing with systems that are inherently event-driven, such as graphical user interfaces. However, it can make it difficult to deliver frame-accurate control over stimulus presentation and is generally harder to understand than traditional, loop-based programming. Nevertheless, it is a useful tool for experiments (or parts of experiments) that don't require precise timing, like questionnaires or other non-timed tasks.

!!! info
    In general, all things that can be done with event-driven programming can also be done with traditional, loop-based programming. See the section on [dealing with events](events.md) for more information.


<div class="grid cards" markdown>
-   Here is a diagram of a single trial in a traditional, **non-**event-driven experiment:

    ---
    ``` mermaid
    flowchart TD
        A[Start of trial] --> B[New frame]
        B --> C[Add stimulus to frame]
        C --> D[Present frame]
        D --> E[Check for response]
        E --> |No response| B
        E --> |Response|F[End trial]
    ```

-   And here is a diagram of a single trial in an event-driven experiment:

    ---
    ``` mermaid
    flowchart TD
        A[Start of trial] --> B[Show stimulus until response]
        B --> |Response|C[End trial]
    ```
</div>


!!! todo
    Add more information about event-driven programming.
