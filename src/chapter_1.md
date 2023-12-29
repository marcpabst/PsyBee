# Chapter 1

# Problem statement - running psychophysics experiments on the web

* The main thread in a browser cannot block. This means that if you run WebAssembly code on the main thread you can never block, meaning you can't do so much as acquire a mutex. This is an extremely difficult limitation to work with on the web, although one workaround is to run wasm exclusively in web workers and run JS on the main thread. It is possible to run the same wasm across all threads, but you need to be extremely vigilant about synchronization with the main thread.
