import scipy as sp
import numpy as np
import matplotlib.pyplot as plt

# target length of filter
N = 1001
fs = 50

# create Gaussian centered at 0, from -N/2 to N/2. assume this is the frequency domain
x = np.arange(-N // 2, N // 2)
F = np.arange(-fs / 2, fs / 2, fs / N)
H = np.exp(-x ** 2 / (2 * 100 ** 2))

# calculate the impulse response using inverse Fourier transform
h = np.fft.ifft(np.fft.ifftshift(H))
h = np.fft.fftshift(h)

# plot the impulse response and the frequency response
fig, ax = plt.subplots(2, 1)
ax[0].plot(h)
ax[0].set_title('Impulse response')
ax[0].set_xlabel('Time (samples)')
ax[0].set_ylabel('Amplitude')

ax[1].plot(F, np.abs(H))
ax[1].set_title('Frequency response')
ax[1].set_xlabel('Frequency (Hz)')
ax[1].set_ylabel('Amplitude')
plt.tight_layout()


# wait for user input
plt.show()
