import numpy as np
from numpy.fft import fft2, ifft2, fftshift, ifftshift

def compute_1D_power_spectrum(image):
    """
    Computes the 1D power spectrum of a 2D array by performing a 2D FFT
    and radially averaging the power spectrum.

    Parameters:
    -----------
    image : 2D numpy array
        Input image or 2D data array.

    Returns:
    --------
    freqs : 1D numpy array
        Frequency values corresponding to the radial profile (in cycles per pixel).
    radial_prof : 1D numpy array
        Radially averaged 1D power spectrum.
    """
    # Compute the 2D FFT of the image
    fft_image = np.fft.fft2(image)
    # Shift the zero frequency component to the center
    fft_image = np.fft.fftshift(fft_image)
    # Compute the power spectrum (magnitude squared of the FFT)
    power_spectrum = np.abs(fft_image)**2

    # Create coordinate grids, centered at zero
    ny, nx = image.shape
    y = np.arange(-ny//2, ny//2)
    x = np.arange(-nx//2, nx//2)
    xv, yv = np.meshgrid(x, y)
    # Compute the radial distances from the center
    r = np.sqrt(xv**2 + yv**2)
    # Flatten arrays
    r = r.flatten()
    ps_flat = power_spectrum.flatten()

    # Get sorted indices
    ind = np.argsort(r)
    r_sorted = r[ind]
    ps_sorted = ps_flat[ind]

    # Bin the data
    r_int = r_sorted.astype(int)
    # Ensure indices are non-negative
    r_int -= r_int.min()
    # Count the number of occurrences in each bin
    bin_counts = np.bincount(r_int)
    # Sum the power spectrum values in each bin
    bin_sums = np.bincount(r_int, weights=ps_sorted)
    # Avoid division by zero
    radial_prof = bin_sums / bin_counts

    # compute the frequency values
    min_freq = 1.0 / max(nx, ny)
    max_freq = 1.0 / 2.0

    freqs = np.linspace(min_freq, max_freq, len(radial_prof))

    return freqs, radial_prof



def log_gabor_filter(image, f0, sigma_f):
    """
    Filters a 2D array using a log-Gabor filter.

    Parameters:
    image   : 2D numpy array representing the image to be filtered.
    f0      : Center frequency of the log-Gabor filter (in cycles per pixel).
    sigma_f : Bandwidth of the filter in log-frequency space.

    Returns:
    filtered_image : 2D numpy array of the filtered image.
    fft_filtered_image : 2D numpy array of the filtered image in the frequency domain.
    log_gabor : 2D numpy array representing the log-Gabor filter.
    """
    rows, cols = image.shape

    # Compute frequency grids
    u = np.fft.fftfreq(cols)
    v = np.fft.fftfreq(rows)
    u = fftshift(u)
    v = fftshift(v)
    u, v = np.meshgrid(u, v)
    radius = np.sqrt(u**2 + v**2)
    radius[radius == 0] = 1e-10  # Avoid division by zero at the origin

    # Log-Gabor filter formula
    log_rad = np.log(radius / f0)
    log_gabor = np.exp(- (log_rad ** 2) / (2 * (np.log(sigma_f / f0) ** 2)))
    log_gabor[radius == 0] = 0  # Suppress the DC component

    # Apply the log-Gabor filter in the frequency domain
    image_fft = fft2(image)
    image_fft_shifted = fftshift(image_fft)
    filtered_fft = image_fft_shifted * log_gabor
    filtered_fft = ifftshift(filtered_fft)
    filtered_image = np.real(ifft2(filtered_fft))

    return filtered_image, filtered_fft, log_gabor

# width and height of the image
w, h = 1500, 1500

# create the 2d noise
noise = np.random.randn(w, h)

# filter the noise
filtered_noise, filtered_fft, filter = log_gabor_filter(noise, f0=0.01, sigma_f=0.015)

# plot the noise and the filtered noise
import matplotlib.pyplot as plt
fig, ax = plt.subplots(2, 3, figsize=(15, 5))

# noise
ax[0,0].imshow(noise, cmap='gray')
ax[0,1].set_title('Noise')
ax[0,2].axis('off')

# log-gabor filter
ax[0,1].imshow(filter, cmap='gray')
ax[0,1].set_title('Filter')
ax[0,1].axis('off')

# filtered noise
ax[0,2].imshow(filtered_noise, cmap='gray')
ax[0,2].set_title('Filtered noise')
ax[0,2].axis('off')

# power spectrum of the noise
ax[1,0].plot(*compute_1D_power_spectrum(noise))
ax[1,0].set_title('Power spectrum of the noise')
ax[1,0].set_xlabel('Frequency')
ax[1,0].set_ylabel('Power')

# power spectrum of the filtered noise
ax[1,2].plot(*compute_1D_power_spectrum(filtered_noise))
ax[1,2].set_title('Power spectrum of the filtered noise')
ax[1,2].set_xlabel('Frequency')
ax[1,2].set_ylabel('Power')

plt.tight_layout()
plt.show()

# tile the images to a 2x2 grid
big_image = np.tile(filtered_noise, (5, 5))

# show the big image
plt.imshow(big_image, cmap='gray')
plt.axis('off')
plt.show()
