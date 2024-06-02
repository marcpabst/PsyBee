import psybee as psy
import time

default_audio_device = psy.AudioDevice()
file_path = "./resources/bubles.mp3"

audio_stim1 = psy.FileStimulus(default_audio_device, file_path)
audio_stim2 = psy.SineWaveStimulus(default_audio_device, 880, 5.0)

while True:

    print("Py: Playing audio_stim2")
    audio_stim1.play()

    # keep the thread alive for 1 second
    time.sleep(1.6)

    print("Py: Resetting audio_stim2")
    audio_stim1.seek(0.1)

    time.sleep(2.5)
